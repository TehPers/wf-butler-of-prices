use crate::models::{Snowflake, User};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub icon: Option<String>,
    pub id: Snowflake,
    pub members: Vec<TeamMember>,
    pub name: String,
    pub owner_user_id: Snowflake,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamMember {
    pub membership_state: MembershipState,
    pub team_id: Snowflake,
    pub user: User,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MembershipState(pub u16);

impl MembershipState {
    pub const INVITED: MembershipState = MembershipState(1);
    pub const ACCEPTED: MembershipState = MembershipState(2);
}
