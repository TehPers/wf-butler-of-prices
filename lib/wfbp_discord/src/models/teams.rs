use crate::{
    models::{User, UserId},
    snowflake_newtype,
};
use serde::{Deserialize, Serialize};

snowflake_newtype! {
    /// A unique ID for a team.
    pub struct TeamId;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub icon: Option<String>,
    pub id: TeamId,
    pub members: Vec<TeamMember>,
    pub name: String,
    pub owner_user_id: UserId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamMember {
    pub membership_state: MembershipState,
    pub team_id: TeamId,
    pub user: User,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MembershipState(pub u16);

impl MembershipState {
    pub const INVITED: MembershipState = MembershipState(1);
    pub const ACCEPTED: MembershipState = MembershipState(2);
}
