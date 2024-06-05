// Forwards compatibility from old operation ctx to new workflows

use std::{fmt::Debug, time::Duration};

use global_error::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::{
	DatabaseHandle, DatabasePostgres, Operation, OperationCtx, OperationInput, Signal, Workflow,
	WorkflowError, WorkflowInput,
};

pub async fn dispatch_workflow<I, B>(
	ctx: &rivet_operation::OperationContext<B>,
	input: I,
) -> GlobalResult<Uuid>
where
	I: WorkflowInput,
	<I as WorkflowInput>::Workflow: Workflow<Input = I>,
	B: Debug + Clone,
{
	if ctx.from_workflow {
		bail!("cannot dispatch a workflow from an operation within a workflow execution. trigger it from the workflow's body.");
	}

	let name = I::Workflow::name();

	tracing::debug!(%name, ?input, "dispatching workflow");

	let id = Uuid::new_v4();

	// Serialize input
	let input_val = serde_json::to_value(input)
		.map_err(WorkflowError::SerializeWorkflowOutput)
		.map_err(GlobalError::raw)?;

	db(ctx)
		.await?
		.dispatch_workflow(ctx.ray_id(), id, &name, input_val)
		.await
		.map_err(GlobalError::raw)?;

	tracing::info!(%name, ?id, "workflow dispatched");

	Ok(id)
}

pub async fn wait_for_workflow<W: Workflow, B: Debug + Clone>(
	ctx: &rivet_operation::OperationContext<B>,
	workflow_id: Uuid,
) -> GlobalResult<W::Output> {
	tracing::info!(name=W::name(), id=?workflow_id, "waiting for workflow");

	let period = Duration::from_millis(50);
	let mut interval = tokio::time::interval(period);
	loop {
		interval.tick().await;

		// Check if state finished
		let workflow = db(ctx)
			.await?
			.get_workflow(workflow_id)
			.await
			.map_err(GlobalError::raw)?
			.ok_or(WorkflowError::WorkflowNotFound)
			.map_err(GlobalError::raw)?;
		if let Some(output) = workflow.parse_output::<W>().map_err(GlobalError::raw)? {
			return Ok(output);
		}
	}
}

pub async fn workflow<I, B>(
	ctx: &rivet_operation::OperationContext<B>,
	input: I,
) -> GlobalResult<<<I as WorkflowInput>::Workflow as Workflow>::Output>
where
	I: WorkflowInput,
	<I as WorkflowInput>::Workflow: Workflow<Input = I>,
	B: Debug + Clone,
{
	let workflow_id = dispatch_workflow(ctx, input).await?;
	let output = wait_for_workflow::<I::Workflow, _>(ctx, workflow_id).await?;
	Ok(output)
}

pub async fn signal<I: Signal + Serialize, B: Debug + Clone>(
	ctx: &rivet_operation::OperationContext<B>,
	workflow_id: Uuid,
	input: I,
) -> GlobalResult<Uuid> {
	if ctx.from_workflow {
		bail!("cannot dispatch a signal from an operation within a workflow execution. trigger it from the workflow's body.");
	}

	tracing::debug!(name=%I::name(), %workflow_id, "dispatching signal");

	let signal_id = Uuid::new_v4();

	// Serialize input
	let input_val = serde_json::to_value(input)
		.map_err(WorkflowError::SerializeSignalBody)
		.map_err(GlobalError::raw)?;

	db(ctx)
		.await?
		.publish_signal(ctx.ray_id(), workflow_id, signal_id, I::name(), input_val)
		.await
		.map_err(GlobalError::raw)?;

	Ok(signal_id)
}

pub async fn op<I, B>(
	ctx: &rivet_operation::OperationContext<B>,
	input: I,
) -> GlobalResult<<<I as OperationInput>::Operation as Operation>::Output>
where
	I: OperationInput,
	<I as OperationInput>::Operation: Operation<Input = I>,
	B: Debug + Clone,
{
	let mut ctx = OperationCtx::new(
		db(ctx).await?,
		ctx.conn(),
		ctx.ray_id(),
		ctx.req_ts(),
		ctx.from_workflow(),
		I::Operation::name(),
	);

	I::Operation::run(&mut ctx, &input)
		.await
		.map_err(WorkflowError::OperationFailure)
		.map_err(GlobalError::raw)
}

// Get crdb pool as a trait object
async fn db<B: Debug + Clone>(
	ctx: &rivet_operation::OperationContext<B>,
) -> GlobalResult<DatabaseHandle> {
	let crdb = ctx.crdb().await?;

	Ok(DatabasePostgres::from_pool(crdb))
}
