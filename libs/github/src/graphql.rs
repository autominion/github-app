use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/query/viewer_info.graphql",
    response_derives = "Debug"
)]
pub struct ViewerInfo;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/query/user_info.graphql",
    response_derives = "Debug"
)]
pub struct UserInfoView;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/query/issue_info.graphql",
    response_derives = "Debug"
)]
pub struct IssueView;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/query/issue_id.graphql",
    response_derives = "Debug"
)]
pub struct IssueIdView;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/mutation/add_comment.graphql",
    response_derives = "Debug"
)]
pub struct AddComment;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/mutation/create_repo.graphql",
    response_derives = "Debug"
)]
pub struct CreateRepo;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/mutation/create_pull_request.graphql",
    response_derives = "Debug"
)]
pub struct CreatePullRequest;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema/schema.graphql",
    query_path = "graphql/query/repo_numeric_id.graphql",
    response_derives = "Debug"
)]
pub struct RepoNumericId;
