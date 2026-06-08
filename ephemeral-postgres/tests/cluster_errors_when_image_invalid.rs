use ephemeral_postgres::cluster::Cluster;
use ephemeral_postgres::cluster_params::ClusterParams;

#[tokio::test]
async fn errors_when_image_tag_cannot_be_resolved() {
    let result = Cluster::start_with_params(ClusterParams {
        image_tag: "tag-that-does-not-exist-9999".to_owned(),
        ..ClusterParams::default()
    })
    .await;

    assert!(
        result.is_err(),
        "start must error when the postgres image tag cannot be resolved",
    );
}
