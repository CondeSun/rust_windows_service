#[cfg(not(windows))]
#[tokio::main]
async fn main() {
    app::run_api_application().await;
}

#[cfg(windows)]
fn main() {
    let _ = win_srv::run();
}

#[cfg(windows)]
mod win_srv {
    use crate::app;
    use std::ffi::OsString;
    use std::time::Duration;
    use tokio::runtime::Runtime;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::{Result, define_windows_service, service_dispatcher};

    static SERVICE_NAME: &'static str = "work_application_service";
    static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        let _ = service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
        define_windows_service!(ffi_service_main, win_srv_main);
        Ok(())
    }

    pub fn win_srv_main(_arguments: Vec<OsString>) {
        if let Err(_e) = run_service() {
            // TODO! Handle Error for Service Impl
        }
    }

    fn run_service() -> Result<()> {
        let rt = Runtime::new().unwrap();
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::unbounded_channel();

        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                ServiceControl::Stop => {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::UserEvent(code) => {
                    if code.to_raw() == 130 {
                        shutdown_tx.send(()).unwrap();
                    }
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        let _result = rt.block_on(async {
            tokio::select! {
                _ = app::run_api_application() => {
                    println!("API wurde von sich aus beendet.");
                },

                _ = shutdown_rx.recv() => {
                    println!("Shutdown-Signal empfangen, beende Anwendung...");
                }
            }
        });

        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }
}

mod app {
    use actix_web::{App, HttpRequest, HttpServer, get};

    #[get("/")]
    async fn index(req: HttpRequest) -> &'static str {
        println!("REQ: {:?}", req);
        "Hello friendWorks!\r\n"
    }

    pub async fn run_api_application() {
        let _res = HttpServer::new(|| App::new().service(index))
            .bind(("127.0.0.1", 8080))
            .expect("Failed to bind on localhost with port 8080")
            .run()
            .await
            .expect("Failed to run actix server");
    }
}
