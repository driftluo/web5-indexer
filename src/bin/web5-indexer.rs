use web5_indexer::{RpcClient, create_pg_pool, did_monitor, get_pg_pool, init_db};

fn main() {
    env_logger::init();
    if std::env::var("ALLOW_EXIT_ON_PANIC")
        .unwrap_or_default()
        .parse()
        .unwrap_or(true)
    {
        std::panic::set_hook(Box::new(|info| {
            log::error!("Panic occurred: {:?}", info);
            std::process::exit(1);
        }));
    }

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async move {
        create_pg_pool().await;
        let pool = get_pg_pool();
        init_db(&pool).await;

        tokio::spawn(async move {
            let rpc = RpcClient::new();
            loop {
                did_monitor(&rpc).await;
                tokio::time::sleep(std::time::Duration::from_secs(10 * 60)).await;
            }
        });

        http_server().await;
    });
}

async fn http_server() {
    use salvo::{
        Listener, Router, Server, Service, conn::TcpListener, cors::AllowOrigin, cors::Cors,
    };
    use web5_indexer::{did_from_addr, did_from_id};

    use salvo::http::Method;
    let cors = Cors::new()
        .allow_origin(AllowOrigin::any())
        .allow_headers(vec!["content-type", "accept", "authorization"])
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .into_handler();

    let router = Router::new()
        .push(Router::with_path("did_from_id").get(did_from_id))
        .push(Router::with_path("did_from_address").get(did_from_addr));

    let service = Service::new(router).hoop(cors);
    let http_port = std::env::var("HTTP_PORT").unwrap_or("8000".to_string());
    let listener = TcpListener::new(format!("0.0.0.0:{}", http_port))
        .bind()
        .await;
    log::info!("Starting HTTP server on port {}", http_port);
    Server::new(listener).serve(service).await;
}
