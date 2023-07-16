use std::sync::Arc;

use actix_web::web;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptyMutation, EmptySubscription, FieldResult, MergedObject, Object, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use service::Service;

#[derive(MergedObject, Default)]
pub struct Query(AttestationQuery);

#[derive(Default)]
pub struct AttestationQuery;

pub type IndexerSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[Object]
impl AttestationQuery {
    async fn participation_rate_for_epoch<'a>(&self, ctx: &'a Context<'_>, epoch: u64) -> FieldResult<f64> {
        let service = ctx.data::<Arc<dyn Service>>()?;
        Ok(service.get_participation_rate_for_epoch(epoch).await?)
    }

    async fn participation_rate_for_validator<'a>(&self, ctx: &'a Context<'_>, validator: u64) -> FieldResult<f64> {
        let service = ctx.data::<Arc<dyn Service>>()?;
        Ok(service.get_participation_rate_for_validator(validator).await?)
    }
}

pub async fn index(schema: web::Data<IndexerSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn index_playground() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}
