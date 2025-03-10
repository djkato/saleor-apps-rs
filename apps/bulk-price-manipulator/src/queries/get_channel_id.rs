#[cynic::schema("saleor")]
mod schema {}
/*
query getChannelID($channel: String!){
  channel(slug: $channel) {
    id
  }
}
*/
#[derive(cynic::QueryVariables, Debug)]
pub struct GetChannelIDVariables<'a> {
    pub channel: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetChannelIDVariables")]
pub struct GetChannelID {
    #[arguments(slug: $channel)]
    pub channel: Option<Channel>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Channel {
    pub id: cynic::Id,
}
