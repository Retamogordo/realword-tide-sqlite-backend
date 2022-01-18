
use once_cell::sync::OnceCell;
use realworld_tide_sqlite_backend::{
    config::Config, 
    backend::*, 
    errors, 
    models::{user, article},
    requests,
    filters,
};

static SERVER: OnceCell<Server> = OnceCell::new();

//#[async_std::test]
async fn connect() -> Result<(), errors::BackendError> {
    let cfg = Config::from_env();

    let mut server = Server::with_config(cfg);
    server.connect().await?;

    SERVER.set(server).expect("Cannot create server instance.");
    Ok(())

}

#[async_std::test]
async fn init() -> Result<(), errors::BackendError> {
    
    let reg_scott_smith = requests::user::UserReg { 
        username: "scott_smith".to_string(),
        email: "scott.smith@fakemail.com".to_string(),
        password: "password".to_string(),
    };

    connect().await?;
    let server = SERVER.get().unwrap();

    let scott_smith_logged_in = server.register_user(reg_scott_smith).await?;

    // login once again just for testing
    let login_scott_smith = filters::UserFilter::default().username("scott_smith"); 
    let scott_smith_logged_in = server.login_user(login_scott_smith).await?;
    // login once again just by email
    let login_scott_smith = filters::UserFilter::default().email("scott.smith@fakemail.com"); 
    let scott_smith_logged_in = server.login_user(login_scott_smith).await?;
    // try logging in non-existing user
    let login_james_joyce = filters::UserFilter::default().username("james_joyce"); 
    server.login_user(login_james_joyce).await.expect_err("Logged in non-existing user.");
    // try logging in non-existing user by email
    let login_james_joyce = filters::UserFilter::default().email("james.joyce@fakemail.com"); 
    server.login_user(login_james_joyce).await.expect_err("Logged in non-existing user.");
    // try registering existing user
    let reg_scott_smith = requests::user::UserReg { 
        username: "scott_smith".to_string(),
        email: "scott.smith@fakemail.com".to_string(),
        password: "password".to_string(),
    };
    server.register_user(reg_scott_smith).await.expect_err("Registered user with username or email already taken.");
    // current user by token
    let scott_smith_current = server.user_by_token(
        scott_smith_logged_in.token.as_ref().unwrap()).await?;
    assert_eq!(scott_smith_current.username, "scott_smith");
    // register and login another user
    let reg_james_joyce = requests::user::UserReg { 
        username: "james_joyce".to_string(),
        email: "james.joyce@fakemail.com".to_string(),
        password: "password".to_string(),
    };

    let james_joyce_logged_in = server.register_user(reg_james_joyce).await?;
    // update user
    let mut set_profile_james_joyce = requests::user::UserUpdateRequest::default();
    let james_joyce_bio = "Im writing a novel";
    set_profile_james_joyce.bio = Some(james_joyce_bio.to_string());
    server.update_user(james_joyce_logged_in.token.as_ref().unwrap(), set_profile_james_joyce)
        .await?;
    // check profile after update
    let james_joyce_profile = server.profile("james_joyce").await?;
    assert_eq!(james_joyce_profile.bio, Some(james_joyce_bio.to_string()));

    server.profile("graham_green").await.expect_err("Got profile of non-existing user.");
    // smith follows joyce
    let james_joyce_profile = server.follow(scott_smith_logged_in.token.as_ref().unwrap(), 
        "james_joyce").await?;
    assert_eq!(james_joyce_profile.username, "james_joyce");
    // james joyce tries to create article
    let create_article = requests::article::CreateArticleRequest { 
        slug: "ulysses".to_string(),
        title: "Ulysses".to_string(), 
        description: None,
        body: "Story not really about traveling".to_string(), 
        tag_list: Some(vec!["Dublin".to_string(), "Homer".to_string()]),
    };
    let article_response1 = server.create_article(
        james_joyce_logged_in.token.as_ref().unwrap(), 
        &create_article).await?;
    
    assert_eq!(james_joyce_profile.username, article_response1.author.unwrap().username);    
    // try creating another article
    let create_article = requests::article::CreateArticleRequest { 
        slug: "finnegans-wake".to_string(),
        title: "Finnegans Wake".to_string(), 
        description: None,
        body: "Some body".to_string(), 
        tag_list: Some(vec!["Dublin".to_string(), "Stream".to_string()]),
    };
    let article_response2 = server.create_article(
        james_joyce_logged_in.token.as_ref().unwrap(), 
        &create_article).await?;
    // test registered tags
    let tags = server.get_tags().await?;
    
    assert_eq!(tags.tags, vec!["Dublin".to_string(), "Homer".to_string(), "Stream".to_string()]);
    // check if articles are in db
    let articles_by_james_joyce = server.get_articles(
        filters::ArticleFilterByValues::default().author("james_joyce".to_string()),
        filters::OrderByFilter::Descending("createdAt"),
        filters::LimitOffsetFilter::default()
    ).await?;

    assert_eq!(2, articles_by_james_joyce.articles.len());

    assert_eq!(articles_by_james_joyce.articles
            .iter()
            .find(|article| article.article.slug == article_response1.article.slug)
            .is_some(),
        true);
    assert_eq!(articles_by_james_joyce.articles
        .iter()
        .find(|article| article.article.slug == article_response2.article.slug)
        .is_some(),
    true);
    // register another user
    let reg_graham_greene = requests::user::UserReg { 
        username: "graham_greene".to_string(),
        email: "graham.greene@fakemail.com".to_string(),
        password: "password".to_string(),
    };

    let graham_greene_logged_in = server.register_user(reg_graham_greene).await?;
    
    let create_article = requests::article::CreateArticleRequest { 
        slug: "the-quiet-american".to_string(),
        title: "The Quiet American".to_string(), 
        description: None,
        body: "Some body".to_string(), 
        tag_list: Some(vec!["Vietnam".to_string(), "Spy".to_string()]),
    };

    let article_response3 = server.create_article(
        graham_greene_logged_in.token.as_ref().unwrap(), 
        &create_article
    ).await?;

    let by_slug = filters::ArticleFilterByValues::default().slug("the-quiet-american".to_string());
    let article_response = server.favorite_article(scott_smith_logged_in.token.as_ref().unwrap(), by_slug)
        .await?;
        assert_eq!(article_response.article.favorited, true);
        assert_eq!(article_response.article.favorites_count, 1);


    
    Ok(())
}