//! These tests do not cover tide requests parsing, instead they 
//! introduce business logic testing steps by artificially forming 
//! middle layer server requests thus simulating local loop without involving tide
//! 
use once_cell::sync::OnceCell;
use async_std::{print, println};
use realworld_tide_sqlite_backend::{
    config::Config, 
    backend::*, 
    errors, 
    models::{article},
    requests,
    filters,
};

static SERVER: OnceCell<Server> = OnceCell::new();

async fn drop_db_and_connect() -> Result<(), errors::BackendError> {
    let mut cfg = Config::from_env();
    cfg.drop_database = true;

    let mut server = Server::with_config(cfg);
    server.connect().await?;

    SERVER.set(server).expect("Cannot create server instance.");
    Ok(())
}

#[async_std::test]
async fn tests() -> Result<(), errors::BackendError> {

    drop_db_and_connect().await?;
    let server = SERVER.get().unwrap();

    let reg_scott_smith = requests::user::UserReg { 
        username: "scott_smith".to_string(),
        email: "scott.smith@fakemail.com".to_string(),
        password: "password".to_string(),
    };

    print!("registering user {} (will take some secs)...", reg_scott_smith.username).await;
    server.register_user(reg_scott_smith).await?;
    println!(" done.").await;

    // login once again just for testing
    let login_scott_smith = requests::user::LoginRequest {
        email: "scott.smith@fakemail.com".to_string(),
        password: "password".to_string(),
    }; 
    print!("logging in user with email {} (will take some secs)...", login_scott_smith.email).await;
    let scott_smith_logged_in = server.login_user(login_scott_smith).await?;
    println!(" done.").await;
    
    // log in with incorrect password
    let login_scott_smith = requests::user::LoginRequest {
        email: "scott.smith@fakemail.com".to_string(),
        password: "incorrect_password".to_string(),
    }; 
    print!("logging in user with incorrect password (will take some secs)...").await;
    server.login_user(login_scott_smith).await.expect_err("Logged in user by incorrect password.");
    println!(" failed as expected.").await;

    // try logging in non-existing user
    let login_james_joyce = requests::user::LoginRequest {
        email: "james.joyce@fakemail.com".to_string(),
        password: "password".to_string(),
    }; 
    print!("logging in non-existing user (will take some secs)...").await;
    server.login_user(login_james_joyce).await.expect_err("Logged in non-existing user.");
    println!(" failed as expected.").await;

    // try registering existing user
    let reg_scott_smith = requests::user::UserReg { 
        username: "scott_smith".to_string(),
        email: "scott.smith@fakemail.com".to_string(),
        password: "password".to_string(),
    };
    print!("trying to register existing user {} (will take some secs)...", reg_scott_smith.username).await;
    server.register_user(reg_scott_smith).await
        .expect_err("Registered user with username or email already taken.");
    println!(" failed as expected.").await;

    // current user by token
    print!("getting user by JWT token ...").await;
    let scott_smith_current = server.user_by_token(
        scott_smith_logged_in.token.as_ref().unwrap()).await?;
    assert_eq!(scott_smith_current.username, "scott_smith");
    println!(" done, username: {}.", scott_smith_current.username).await;

    // register and login another user
    let reg_james_joyce = requests::user::UserReg { 
        username: "james_joyce".to_string(),
        email: "james.joyce@fakemail.com".to_string(),
        password: "password".to_string(),
    };
    print!("registering user {} (will take some secs)...", reg_james_joyce.username).await;
    let james_joyce_logged_in = server.register_user(reg_james_joyce).await?;
    println!(" done.").await;
    
    // update user
    let mut set_profile_james_joyce = requests::user::UserUpdateRequest::default();
    let james_joyce_bio = "Im writing a novel";
    set_profile_james_joyce.bio = Some(james_joyce_bio.to_string());
    print!("updating users {} profile...", james_joyce_logged_in.username).await;
    let profile_james_joyce = server.update_user(james_joyce_logged_in.token.as_ref().unwrap(), set_profile_james_joyce)
        .await?;
    assert_eq!(profile_james_joyce.bio, james_joyce_bio.to_string());
    println!(" done, updated bio: {}", profile_james_joyce.bio).await;

    //
    print!("retrieving non-existing users profile...").await;
    server.profile("graham_green").await.expect_err("Got profile of non-existing user.");
    println!(" failed as expected.").await;
    
    // smith follows joyce
    print!("{} follows user {} ...", scott_smith_logged_in.username, james_joyce_logged_in.username).await;
    let james_joyce_profile = server.follow(scott_smith_logged_in.token.as_ref().unwrap(), 
        "james_joyce").await?;
    assert_eq!(james_joyce_profile.username, "james_joyce");
    println!(" done, followed user is {}.", profile_james_joyce.username).await;

    // james joyce tries to create article
    let create_article = requests::article::CreateArticleRequest { 
        slug: "ulysses".to_string(),
        title: "Ulysses".to_string(), 
        description: None,
        body: "Story not really about traveling".to_string(), 
        tag_list: Some(vec!["Dublin".to_string(), "Homer".to_string()]),
    };
    print!("{} creates an article ...", james_joyce_logged_in.username).await;
    let article_response1 = server.create_article(
        james_joyce_logged_in.token.as_ref().unwrap(), 
        create_article).await?;
    
    assert_eq!(james_joyce_profile.username, article_response1.author.as_ref().unwrap().username);    
    println!(" done, title: {}.", article_response1.article.title).await;
    
    // try creating another article
    print!("{} creates an article ...", james_joyce_logged_in.username).await;
    let create_article = requests::article::CreateArticleRequest { 
        slug: "finnegans-wake".to_string(),
        title: "Finnegans Wake".to_string(), 
        description: None,
        body: "Some body".to_string(), 
        tag_list: Some(vec!["Dublin".to_string(), "Stream".to_string()]),
    };
    let article_response2 = server.create_article(
        james_joyce_logged_in.token.as_ref().unwrap(), 
        create_article).await?;
    println!(" done, title: {}.", article_response2.article.title).await;
    // test registered tags
    print!("getting tags ...").await;
    let tags = server.get_tags().await?;
    assert_eq!(tags.tags, vec!["Dublin".to_string(), "Homer".to_string(), "Stream".to_string()]);
    println!(" done, they are: {}.", tags).await;
    
    // check if articles are in db
    print!("checking number of registered articles...").await;
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
    println!(" done.").await;
    
    // register another user
    let reg_graham_greene = requests::user::UserReg { 
        username: "graham_greene".to_string(),
        email: "graham.greene@fakemail.com".to_string(),
        password: "password".to_string(),
    };
    print!("registering user {}...", reg_graham_greene.username).await;
    let graham_greene_logged_in = server.register_user(reg_graham_greene).await?;
    println!(" done.").await;
    
    let create_article = requests::article::CreateArticleRequest { 
        slug: "the-quiet-american".to_string(),
        title: "The Quiet American".to_string(), 
        description: None,
        body: "Some body".to_string(), 
        tag_list: Some(vec!["Vietnam".to_string(), "Spy".to_string()]),
    };
    print!("{} creates an article ...", graham_greene_logged_in.username).await;
    let article_response3 = server.create_article(
        graham_greene_logged_in.token.as_ref().unwrap(), 
        create_article
    ).await?;
    println!(" done, title: {}.", article_response3.article.title).await;

    // scott smith tries to favorite article by greene
    let slug ="the-quiet-american";
    print!("{} favorites an article {}...", scott_smith_logged_in.username, slug).await;
    let article_response = server.favorite_article(scott_smith_logged_in.token.as_ref().unwrap(), 
        slug)
        .await?;
    assert_eq!(article_response.article.favorited, true);
    assert_eq!(article_response.article.favorites_count, 1);
    println!(" done.").await;

    let slug = "the-quiet-american";
    // james joyce tries to favorite article by greene
    print!("{} favorites article {}...", james_joyce_logged_in.username, slug).await;
    let article_response = server.favorite_article(james_joyce_logged_in.token.as_ref().unwrap(), 
        slug)
        .await?;
    assert_eq!(article_response.article.favorited, true);
    assert_eq!(article_response.article.favorites_count, 2);
    println!(" done.").await;

    let slug = "the-quiet-american";
    print!("{} unfavorites article {}...", james_joyce_logged_in.username, slug).await;
    let article_response = server.unfavorite_article(james_joyce_logged_in.token.as_ref().unwrap(), 
        slug)
        .await?;
    assert_eq!(article_response.article.favorited, true);
    assert_eq!(article_response.article.favorites_count, 1);
    println!(" done.").await;

    // smith follows greene
    print!("{} follows user {}...", scott_smith_logged_in.username, graham_greene_logged_in.username).await;
    let graham_green_profile = server.follow(scott_smith_logged_in.token.as_ref().unwrap(), 
        "graham_greene").await?;
    assert_eq!(graham_green_profile.username, "graham_greene");
    println!(" done.").await;
    
    // feeds articles for smith
    print!("feed articles for user {}...", scott_smith_logged_in.username).await;
    let articles_fed_for_scott_smith = server.feed_articles(
        scott_smith_logged_in.token.as_ref().unwrap(), 
        filters::LimitOffsetFilter::default(),
    )
    .await?; 
    assert_eq!(3, articles_fed_for_scott_smith.articles.len());
    println!(" done, fed {} articles.", articles_fed_for_scott_smith.articles.len()).await;
    
    // greene tries to update his article's title
    let update_article_req = requests::article::UpdateArticleRequest {
        article: article::UpdateArticle::default().title("The Calm American"),
        slug: "the-quiet-american",
    };
    print!("{} updates article {}...", graham_greene_logged_in.username, update_article_req.slug).await;
    server.update_article(
        graham_greene_logged_in.token.as_ref().unwrap(), 
        update_article_req,
    )
    .await?;
    println!(" done.").await;
    // check if the old slug is absent
    let articles_by_graham_green = server.get_articles(
        filters::ArticleFilterByValues::default().slug("the-quiet-american".to_string()),
        filters::OrderByFilter::Descending("createdAt"),
        filters::LimitOffsetFilter::default()
    ).await?;
    assert_eq!(0, articles_by_graham_green.articles.len());

    // check if the new slug is there
    let articles_by_graham_green = server.get_articles(
        filters::ArticleFilterByValues::default().slug("the-calm-american".to_string()),
        filters::OrderByFilter::Descending("createdAt"),
        filters::LimitOffsetFilter::default()
    ).await?;
    assert_eq!(1, articles_by_graham_green.articles.len());

    // try forbidden article deleteting
    print!("{} deletes article {}...", scott_smith_logged_in.username, "the-calm-american").await;
    server.delete_article(
        scott_smith_logged_in.token.as_ref().unwrap(), 
        "the-calm-american",
    )
    .await
    .expect_err("User was not authorized to delete article not belonging to him");
    println!(" failed for forbidden.").await;

    // try forbidden article updating
    let update_article_req = requests::article::UpdateArticleRequest {
        article: article::UpdateArticle::default().title("The Mad American"),
        slug: "the-calm-american",
    };
    print!("{} updates article {}...", scott_smith_logged_in.username, update_article_req.slug).await;    
    server.update_article(
        scott_smith_logged_in.token.as_ref().unwrap(), 
        update_article_req,
    )
    .await
    .expect_err("User was not authorized to update article not belonging to him");
    println!(" failed for forbidden.").await;

    // try deleting non-existing article
    print!("{} deletes non-existing article ...", scott_smith_logged_in.username).await;
    server.delete_article(
        scott_smith_logged_in.token.as_ref().unwrap(), 
        "dummy",
    )
    .await
    .expect_err("Succeded to delete non-existing article");
    println!(" failed as expected.").await;

    // try updating non-existing article
    let update_article_req = requests::article::UpdateArticleRequest {
        article: article::UpdateArticle::default().title("The Quiet American"),
        slug: "the-good-american",
    };
    print!("{} updates non-existing article ...", scott_smith_logged_in.username).await;
    server.update_article(
        graham_greene_logged_in.token.as_ref().unwrap(), 
        update_article_req,
    )
    .await
    .expect_err("Succeded to update non-existing article");
    println!(" failed as expected.").await;

    // add first comment
    let comment_req = requests::article::AddCommentRequest {
        article_slug: "the-calm-american",
        body: "Author, write more !".to_string(),
    };

    print!("{} adds a comment to article {}...", scott_smith_logged_in.username, 
        comment_req.article_slug).await;
    server.add_comment(
        scott_smith_logged_in.token.as_ref().unwrap(), 
        comment_req
    )
    .await?;
    println!(" done.").await;
    // add second comment
    let comment_req = requests::article::AddCommentRequest {
        article_slug: "the-calm-american",
        body: "Hmmm...".to_string(),
    };
    print!("{} adds a comment to article {}...", scott_smith_logged_in.username, 
        comment_req.article_slug).await;
    server.add_comment(
        scott_smith_logged_in.token.as_ref().unwrap(), 
        comment_req
    )
    .await?;
    println!(" done.").await;

    let comment_req = requests::article::AddCommentRequest {
        article_slug: "the-calm-american",
        body: "Should call it The Quiet American".to_string(),
    };
    print!("{} adds a comment to article {}...", james_joyce_logged_in.username, 
        comment_req.article_slug).await;
    server.add_comment(
        james_joyce_logged_in.token.as_ref().unwrap(), 
        comment_req
    )
    .await?;
    println!(" done.").await;
    
    let comment_req = requests::article::AddCommentRequest {
        article_slug: "the-calm-american",
        body: "I love all my writings".to_string(),
    };
    print!("{} adds a comment to article {}...", graham_greene_logged_in.username, 
        comment_req.article_slug).await;
    let last_comment = server.add_comment(
        graham_greene_logged_in.token.as_ref().unwrap(), 
        comment_req
    )
    .await?;
    assert_eq!(last_comment.author.unwrap().username, "graham_greene");
    println!(" done.").await;

    let article_slug = "the-calm-american";
    
    let comments = server.get_comments(article_slug).await?;
    assert_eq!(4, comments.comments.len());

    // delete comment
    let delete_by = requests::article::DeleteCommentRequest {
        id: 4,
        article_slug: "the-calm-american",
    };
    print!("{} deletes a comment to article {}...", graham_greene_logged_in.username, 
        delete_by.article_slug).await;
    server.delete_comment(
        graham_greene_logged_in.token.as_ref().unwrap(), 
        delete_by,
    )
    .await?;
    println!(" done.").await;

    // check if comment has been really deleted
    let article_slug = "the-calm-american";

    let comments = server.get_comments(article_slug).await?;
    assert_eq!(3, comments.comments.len());

    // delete non-existing comment
    let delete_by = requests::article::DeleteCommentRequest {
        id: 3,
        article_slug: "the-calm-american",
    };
    print!("{} deletes a (not owned) comment to article {}...", 
        graham_greene_logged_in.username, delete_by.article_slug).await;
    server.delete_comment(
        graham_greene_logged_in.token.as_ref().unwrap(), 
        delete_by,
    )
    .await
    .expect_err("Deleting non-belonging comment.");
    println!(" failed as expected.").await;

    Ok(())
}