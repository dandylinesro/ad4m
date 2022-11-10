use graphql_client::{GraphQLQuery};
use graphql_ws_client::graphql::StreamingOperation;
use serde_json::Value;
use crate::util::{create_websocket_client, query};
use anyhow::{Result, Context};
use chrono::naive::NaiveDateTime;

type DateTime = NaiveDateTime;

use self::all::AllPerspectives;
use self::add_link::AddLinkPerspectiveAddLink;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../core/lib/src/schema.gql",
    query_path = "src/perspectives.gql",
    response_derives = "Debug",
)]
pub struct All;

pub async fn run_all(cap_token: String) -> Result<Vec<AllPerspectives>> {
    let response_data: all::ResponseData = query(cap_token, All::build_query(all::Variables {}))
        .await
        .with_context(|| "Failed to run perspectives->all query")?;
    Ok(response_data.perspectives)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../core/lib/src/schema.gql",
    query_path = "src/perspectives.gql",
    response_derives = "Debug",
)]
pub struct Add;

pub async fn run_add(cap_token: String, name: String) -> Result<String> {
    let response_data: add::ResponseData = query(cap_token, Add::build_query(add::Variables { name }))
        .await
        .with_context(|| "Failed to run perspectives->add query")?;
    Ok(response_data.perspective_add.uuid)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../core/lib/src/schema.gql",
    query_path = "src/perspectives.gql",
    response_derives = "Debug",
)]
pub struct Remove;

pub async fn run_remove(cap_token: String, uuid: String) -> Result<()> {
    query(cap_token, Remove::build_query(remove::Variables { uuid }))
        .await
        .with_context(|| "Failed to run perspectives->remove query")?;
    Ok(())
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../core/lib/src/schema.gql",
    query_path = "src/perspectives.gql",
    response_derives = "Debug",
)]
pub struct AddLink;

pub async fn run_add_link(cap_token: String, uuid: String, source: String, target: String, predicate: Option<String>) -> Result<AddLinkPerspectiveAddLink> {
    let response_data: add_link::ResponseData = query(
        cap_token, 
        AddLink::build_query(add_link::Variables { 
            uuid, 
            link: add_link::LinkInput {
                source,
                target,
                predicate,
            }
        })
    )
        .await
        .with_context(|| "Failed to run perspectives->addLink query")?;
    
    Ok(response_data.perspective_add_link)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../core/lib/src/schema.gql",
    query_path = "src/perspectives.gql",
    response_derives = "Debug",
)]
pub struct QueryLinks;

pub async fn run_query_links(
    cap_token: String, 
    uuid: String, 
    source: Option<String>, 
    target: Option<String>, 
    predicate: Option<String>,
    from_date: Option<DateTime>,
    until_date: Option<DateTime>,
    limit: Option<f64>
) -> Result<Vec<query_links::QueryLinksPerspectiveQueryLinks>> {

    let response_data: query_links::ResponseData = query(
        cap_token, 
        QueryLinks::build_query(query_links::Variables { 
            uuid,
            query: query_links::LinkQuery {
                source, 
                target, 
                predicate,
                fromDate: from_date,
                untilDate: until_date,
                limit,
            }
        })
    )
        .await
        .with_context(|| "Failed to run perspectives->queryLinks query")?;

    Ok(response_data.perspective_query_links.unwrap_or_default())
}
 

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../core/lib/src/schema.gql",
    query_path = "src/perspectives.gql",
    response_derives = "Debug",
)]
pub struct Infer;

pub async fn run_infer(cap_token: String, uuid: String, prolog_query: String) -> Result<Value> {
    let response_data: infer::ResponseData = query(cap_token, Infer::build_query(infer::Variables { uuid, query: prolog_query }))
        .await
        .with_context(|| "Failed to run perspectives->infer query")?;
    let v: Value = serde_json::from_str(&response_data.perspective_query_prolog)?;
    Ok(match v {
        Value::String(string) => {
            if string == "true" {
                Value::Bool(true)
            } else if string == "false" {
                Value::Bool(false)
            } else {
                Value::String(string)
            }
        },
        _ => v,
    })
}

struct TokioSpawner(tokio::runtime::Handle);

impl TokioSpawner {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        TokioSpawner(handle)
    }

    pub fn current() -> Self {
        TokioSpawner::new(tokio::runtime::Handle::current())
    }
}

impl futures::task::Spawn for TokioSpawner {
    fn spawn_obj(
        &self,
        obj: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        self.0.spawn(obj);
        Ok(())
    }
}


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../core/lib/src/schema.gql",
    query_path = "src/perspectives.gql",
    response_derives = "Debug",
)]
pub struct SubscriptionLinkAdded;

pub async fn run_watch(cap_token: String, id: String) -> Result<()> {
    use futures::StreamExt;

    let mut client = create_websocket_client(cap_token).await?;
    let mut stream = client.streaming_operation(StreamingOperation::<SubscriptionLinkAdded>::new(subscription_link_added::Variables {
        uuid: id,
    })).await?;
    println!("Running subscription apparently?");
    while let Some(item) = stream.next().await {
        println!("{:?}", item);
    }

    println!("after loop");

    Ok(())
}