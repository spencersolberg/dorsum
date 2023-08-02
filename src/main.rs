use std::net::SocketAddr;

use axum::{
    body::Body,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use std::process::Command;
use tower_http::services::ServeFile;
use serde_json::Value;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let routes_all = Router::new().merge(routes());

    let address: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on http://{}", address);
    axum::Server::bind(&address)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
}

fn routes() -> Router {
    Router::new()
        .route("/", get(handler_index))
        .route("/certificates", get(handler_certificates))
        .route("/ios", get(handler_ios))
        .route("/tailscale", get(handler_tailscale))
        .route("/ios/tailscale-dot.mobileconfig", get(handler_tailscale_dot))
        .route("/ios/tailscale-doh.mobileconfig", get(handler_tailscale_doh))
        .route("/ios/tailscale-proxy.mobileconfig", get(handler_tailscale_proxy))
        .merge(routes_certificates())
}

fn routes_certificates() -> Router {
    Router::new()
        .nest_service(
            "/certificates/dorsum-root.crt",
            get_service(ServeFile::new("/etc/dorsum/certificates/dorsum-root.crt")),
        )
        .nest_service(
            "/certificates/letsdane.crt",
            get_service(ServeFile::new("/etc/dorsum/certificates/letsdane.crt")),
        )
}

async fn handler_index() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}

async fn handler_certificates() -> impl IntoResponse {
    Html(include_str!("../static/certificates.html"))
}

async fn handler_ios() -> impl IntoResponse {
    Html(include_str!("../static/ios.html"))
}

async fn handler_tailscale_dot() -> impl IntoResponse {
    let output = Command::new("tailscale")
        .arg("status")
        .arg("--json")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8(output.stdout).expect("Found invalid UTF-8");

    let v: Value = serde_json::from_str(&output_str).expect("JSON was not well-formatted");

    let mobileconfig = format!("
        <plist version='1.0'>
        <dict>
        <key>PayloadContent</key>
        <array>
        <dict>
        <key>DNSSettings</key>
        <dict>
        <key>DNSProtocol</key>
        <string>TLS</string>
        <key>ServerAddresses</key>
        <array/>
        <key>ServerName</key>
        <string>{}</string>
        </dict>
        <key>OnDemandRules</key>
        <array>
        <dict>
        <key>Action</key>
        <string>Connect</string>
        <key>InterfaceTypeMatch</key>
        <string>WiFi</string>
        </dict>
        <dict>
        <key>Action</key>
        <string>Connect</string>
        <key>InterfaceTypeMatch</key>
        <string>Cellular</string>
        </dict>
        <dict>
        <key>Action</key>
        <string>Disconnect</string>
        </dict>
        </array>
        <key>PayloadDescription</key>
        <string>Configures device to use dorsum Encrypted DNS over TLS</string>
        <key>PayloadDisplayName</key>
        <string>dorsum DNS over TLS</string>
        <key>PayloadIdentifier</key>
        <string>com.apple.dnsSettings.managed.{}</string>
        <key>PayloadType</key>
        <string>com.apple.dnsSettings.managed</string>
        <key>PayloadUUID</key>
        <string>{}</string>
        <key>PayloadVersion</key>
        <integer>1</integer>
        <key>ProhibitDisablement</key>
        <false/>
        </dict>
        </array>
        <key>PayloadDescription</key>
        <string>Adds different encrypted DNS configurations to Big Sur (or newer) and iOS 14 (or newer) based systems</string>
        <key>PayloadDisplayName</key>
        <string>Encrypted DNS (DoH, DoT)</string>
        <key>PayloadIdentifier</key>
        <string>sh.dorsum.apple-dns.{}</string>
        <key>PayloadRemovalDisallowed</key>
        <false/>
        <key>PayloadType</key>
        <string>Configuration</string>
        <key>PayloadUUID</key>
        <string>{}</string>
        <key>PayloadVersion</key>
        <integer>1</integer>
        </dict>
        </plist>
    ",
    v["TailscaleIPs"][0].as_str().unwrap(),
    // 4 uuids
    Uuid::new_v4(),
    Uuid::new_v4(),
    Uuid::new_v4(),
    Uuid::new_v4(),
);

    // return mobileconfig as application/x-apple-aspen-config response

    let body = Body::from(mobileconfig);
    Response::builder()
        .header("Content-Type", "application/x-apple-aspen-config")
        .body(body)
        .unwrap()

}

async fn handler_tailscale_doh() -> impl IntoResponse {
    let output = Command::new("tailscale")
    .arg("status")
    .arg("--json")
    .output()
    .expect("Failed to execute command");

    let output_str = String::from_utf8(output.stdout).expect("Found invalid UTF-8");

    let v: Value = serde_json::from_str(&output_str).expect("JSON was not well-formatted");

    let mobileconfig = format!("
        <plist version='1.0'>
        <dict>
        <key>PayloadContent</key>
        <array>
        <dict>
        <key>DNSSettings</key>
        <dict>
        <key>DNSProtocol</key>
        <string>HTTPS</string>
        <key>ServerAddresses</key>
        <array/>
        <key>ServerURL</key>
        <string>{}/query</string>
        </dict>
        <key>OnDemandRules</key>
        <array>
        <dict>
        <key>Action</key>
        <string>Connect</string>
        <key>InterfaceTypeMatch</key>
        <string>WiFi</string>
        </dict>
        <dict>
        <key>Action</key>
        <string>Connect</string>
        <key>InterfaceTypeMatch</key>
        <string>Cellular</string>
        </dict>
        <dict>
        <key>Action</key>
        <string>Disconnect</string>
        </dict>
        </array>
        <key>PayloadDescription</key>
        <string>Configures device to use dorsum Encrypted DNS over HTTPS</string>
        <key>PayloadDisplayName</key>
        <string>dorsum DNS over HTTPS</string>
        <key>PayloadIdentifier</key>
        <string>com.apple.dnsSettings.managed.{}</string>
        <key>PayloadType</key>
        <string>com.apple.dnsSettings.managed</string>
        <key>PayloadUUID</key>
        <string>{}</string>
        <key>PayloadVersion</key>
        <integer>1</integer>
        <key>ProhibitDisablement</key>
        <false/>
        </dict>
        </array>
        <key>PayloadDescription</key>
        <string>Adds different encrypted DNS configurations to Big Sur (or newer) and iOS 14 (or newer) based systems</string>
        <key>PayloadDisplayName</key>
        <string>Encrypted DNS (DoH, DoT)</string>
        <key>PayloadIdentifier</key>
        <string>sh.dorsum.apple-dns.{}</string>
        <key>PayloadRemovalDisallowed</key>
        <false/>
        <key>PayloadType</key>
        <string>Configuration</string>
        <key>PayloadUUID</key>
        <string>{}</string>
        <key>PayloadVersion</key>
        <integer>1</integer>
        </dict>
        </plist>
    ",
    v["TailscaleIPs"][0].as_str().unwrap(),
    // 4 uuids
    Uuid::new_v4(),
    Uuid::new_v4(),
    Uuid::new_v4(),
    Uuid::new_v4(),
    );

    let body = Body::from(mobileconfig);
    Response::builder()
        .header("Content-Type", "application/x-apple-aspen-config")
        .body(body)
        .unwrap()

}

async fn handler_tailscale_proxy() -> impl IntoResponse {
    let output = Command::new("tailscale")
    .arg("status")
    .arg("--json")
    .output()
    .expect("Failed to execute command");

    let output_str = String::from_utf8(output.stdout).expect("Found invalid UTF-8");

    let v: Value = serde_json::from_str(&output_str).expect("JSON was not well-formatted");

    let mobileconfig = format!("
    <?xml version='1.0' encoding='UTF-8'?>
    <!DOCTYPE plist PUBLIC '-//Apple//DTD PLIST 1.0//EN' 'http://www.apple.com/DTDs/PropertyList-1.0.dtd'>
    <plist version='1.0'>
    <dict>
        <key>PayloadContent</key>
        <array>
            <dict>
                <key>ProxyCaptiveLoginAllowed</key>
                <true/>
                <key>ProxyServer</key>
                <string>{}</string>
                <key>ProxyServerPort</key>
                <integer>8080</integer>
                <key>PayloadIdentifier</key>
                <string>sh.dorsum.proxy</string>
                <key>PayloadType</key>
                <string>com.apple.proxy.http.global</string>
                <key>PayloadUUID</key>
                <string>{}</string>
                <key>PayloadVersion</key>
                <integer>1</integer>
            </dict>
        </array>
        <key>PayloadDisplayName</key>
        <string>GlobalHTTPProxy</string>
        <key>PayloadIdentifier</key>
        <string>com.example.myprofile</string>
        <key>PayloadType</key>
        <string>Configuration</string>
        <key>PayloadUUID</key>
        <string>{}</string>
        <key>PayloadVersion</key>
        <integer>1</integer>
    </dict>
    </plist>
    ",
    v["TailscaleIPs"][0].as_str().unwrap(),
    // 2 uuids
    Uuid::new_v4(),
    Uuid::new_v4(),
    );

    let body = Body::from(mobileconfig);

    Response::builder()
        .header("Content-Type", "application/x-apple-aspen-config")
        .body(body)
        .unwrap()
}

async fn handler_tailscale() -> impl IntoResponse {
    // run $ tailscale status --json

    let output = Command::new("tailscale")
        .arg("status")
        .arg("--json")
        .output()
        .expect("Failed to execute command");

    // let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    let output_str = String::from_utf8(output.stdout).expect("Found invalid UTF-8");

    let v: Value = serde_json::from_str(&output_str).expect("JSON was not well-formatted");

    let status_color = match v["BackendState"].as_str().unwrap() {
        "Running" => "green",
        "Starting" => "yellow",
        "Stopped" => "red",
        _ => "red",
    };

    Html(format!("
    <style>
        .status {{
            color: {};
        }}
    </style>
    <h1>dorsum</h1>
    <h3>tailscale</h3>
    <a href='/'>go back</a><br />
    <span>status - <span class='status'>{}</span></span><br />
    <span>ipv4 - {}</span><br />
    ", status_color, v["BackendState"].as_str().unwrap(), v["TailscaleIPs"][0].as_str().unwrap()))

}
