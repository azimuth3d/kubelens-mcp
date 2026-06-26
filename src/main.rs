mod mcp;
mod tools;
mod cluster;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let use_mock = std::env::var("KUBELENS_USE_MOCK").unwrap_or_default() == "true";
    let cluster: Box<dyn cluster::traits::ClusterDiagnostics> = if use_mock {
        Box::new(cluster::mock_client::MockClusterClient)
    } else {
        match cluster::kube_client::KubeSdkAdapter::new().await {
            Ok(adapter) => Box::new(adapter),
            Err(e) => {
                eprintln!("Failed to initialize live cluster client: {}. Falling back to mock.", e);
                Box::new(cluster::mock_client::MockClusterClient)
            }
        }
    };

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut writer = tokio::io::stdout();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                if line.trim().is_empty() { continue; }
                
                if let Some(response_json) = mcp::handle_request(&line, cluster.as_ref()).await {
                    writer.write_all(response_json.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                }
            }
            Err(e) => {
                eprintln!("IO Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
