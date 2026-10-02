use serde_json::json;

#[tokio::test]
async fn test_backend() {
    let hc = httpc_test::new_client("http://localhost:3000")
        .expect("failed to connect to the testing environment");

    hc.do_get("/me").await.unwrap().print().await.unwrap();
    //hc.do_get("/health").await.unwrap().print().await.unwrap();
    // hc.do_get("/db-check").await.unwrap().print().await.unwrap();

    let _req_register = hc.do_post(
        "/register",
        json!({
            "username":"lucy",
            "password":"hunter2"
        }),
    );
    //req_register.await.unwrap().print().await.unwrap();

    hc.do_get("/todos").await.unwrap().print().await.unwrap();

    let req_login = hc.do_post(
        "/login",
        json!({
            "username":"lucy",
            "password":"hunter2"
        }),
    );
    req_login.await.unwrap().print().await.unwrap();
    hc.do_get("/me").await.unwrap().print().await.unwrap();

    let _req_post = hc.do_post(
        "/todos",
        json!({
            "title":"learn rust"
        }),
    );
    //req_post.await.unwrap().print().await.unwrap();

    let req_update = hc.do_patch(
        "/posts/39d5a4a7-d424-4461-aacc-709bf784a534",
        json!({
            "title":"learn blockchain"
        }),
    );
    //req_update.await.unwrap().print().await.unwrap();

    hc.do_get("/todos").await.unwrap().print().await.unwrap();
}
