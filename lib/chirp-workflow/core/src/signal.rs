use async_trait::async_trait;

use crate::{WorkflowCtx, WorkflowResult};

pub trait Signal {
	fn name() -> &'static str;
}

/// A trait which allows listening for signals from the workflows database. This is used by
/// `WorkflowCtx::listen` and `WorkflowCtx::query_signal`.
#[async_trait]
pub trait Listen: Sized {
	async fn listen(ctx: &mut WorkflowCtx) -> WorkflowResult<Self>;
	fn parse(name: &str, body: serde_json::Value) -> WorkflowResult<Self>;
}

/// Creates an enum that implements `Listen` and selects one of X signals.
///
/// Example:
/// ```rust
/// #[macros::signal("my-signal")]
/// struct MySignal {
/// 	x: i64,
/// }

/// #[macros::signal("my-signal2")]
/// struct MySignal2 {
/// 	y: i64,
/// }
///
/// join_signal!(MyJoinSignal, [MySignal, MySignal2]);
///
/// // Automatically becomes:
/// enum MyJoinSignal {
/// 	MySignal(MySignal),
/// 	MySignal2(MySignal2),
/// }
///
/// // Listening:
/// match ctx.listen::<MyJoinSignal>() {
/// 	MySignal(sig) => println!("received MySignal {sig:?}"),
/// 	MySignal2(sig) => println!("received MySignal2 {sig:?}"),
/// }
/// ````
#[macro_export]
macro_rules! join_signal {
	($join:ident, [$($signals:ident),*]) => {
		enum $join {
			$($signals($signals)),*
		}

		#[::async_trait::async_trait]
		impl Listen for $join {
			async fn listen(ctx: &mut chirp_workflow::prelude::WorkflowCtx) -> chirp_workflow::prelude::WorkflowResult<Self> {
				let row = ctx.listen_any(&[$($signals::name()),*]).await?;
				Self::parse(&row.name, row.body)
			}

			fn parse(name: &str, body: serde_json::Value) -> chirp_workflow::prelude::WorkflowResult<Self> {
				$(
					if name == $signals::name() {
						Ok(
							Self::$signals(
								serde_json::from_value(body)
									.map_err(WorkflowError::DeserializeActivityOutput)?
							)
						)
					}
				)else*

				else {
					unreachable!("received signal that wasn't queried for");
				}
			}
		}
	}
}
