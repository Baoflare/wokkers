/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ActorPortRouting {
<<<<<<< HEAD
    #[serde(rename = "guard", skip_serializing_if = "Option::is_none")]
    pub guard: Option<Box<crate::models::ActorGuardRouting>>,
=======
    #[serde(rename = "game_guard", skip_serializing_if = "Option::is_none")]
    pub game_guard: Option<Box<crate::models::ActorGameGuardRouting>>,
>>>>>>> 73a068837 (feat: revamp actor build endpoint, js builds -> tar)
    #[serde(rename = "host", skip_serializing_if = "Option::is_none")]
    pub host: Option<serde_json::Value>,
}

impl ActorPortRouting {
    pub fn new() -> ActorPortRouting {
        ActorPortRouting {
<<<<<<< HEAD
            guard: None,
=======
            game_guard: None,
>>>>>>> 73a068837 (feat: revamp actor build endpoint, js builds -> tar)
            host: None,
        }
    }
}


