use anyhow::Result;
use rpc::ApiClient;
use tracing::info;

#[cfg(target_arch = "wasm32")]
pub async fn get_client() -> Result<ApiClient> {
    use crate::rpc_client_wasm::*;
    use wasm_bindgen_futures::spawn_local;
    use anyhow::anyhow;
    let transport = build_client();
    transport.await.map(|trans| {
        let config = tarpc::client::Config::default();
        let client = ApiClient::new(config, trans);
        let dispatch = client
            .dispatch;
        info!("Spawning Dispatch");
        spawn_local(async move { dispatch.await.unwrap(); });
        client.client
    }).map_err(|e| anyhow!(e.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_client() -> Result<ApiClient> {
    use tokio_serde::formats::Json;
    use tarpc::client;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let transport = tarpc::serde_transport::tcp::connect(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), rpc::API_PORT), Json::default);
    let client = ApiClient::new(client::Config::default(), transport.await?).spawn();
    info!("connected");
    Ok(client)
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use crate::rpc_client::get_client;

    #[test]
    fn client_test() -> Result<()> {
        tokio::runtime::Builder::new_current_thread()
            .enable_io().build()?.block_on(async {
            let client = get_client().await.unwrap();
            let a = client.ping(tarpc::context::current()).await.unwrap().unwrap();
            println!("result: {:?}", a);
        });
        Ok(())
    }
}