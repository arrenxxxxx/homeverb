use super::*;

struct ExpiredRead {
    wire: Arc<Wire>,
    reads: AtomicUsize,
    logins: AtomicUsize,
    failure: &'static str,
}
#[async_trait]
impl CloudTransport for ExpiredRead {
    async fn send(&self, request: &SignedRequest, operation: &Operation) -> Result<SecretBytes> {
        match request.endpoint() {
            CloudEndpoint::LoginId => {
                self.logins.fetch_add(1, Ordering::Relaxed);
                Ok(bytes(json!({"code":0,"data":{"loginId":"FIRST"}})))
            }
            CloudEndpoint::Login => {
                self.logins.fetch_add(1, Ordering::Relaxed);
                let vector: Value =
                    serde_json::from_str(include_str!("../fixtures/login.json")).unwrap();
                Ok(bytes(
                    json!({"code":0,"data":{"mdata":{"accessToken":"FIRST"},"key":vector["wrapped_key"]}}),
                ))
            }
            CloudEndpoint::ApplianceState => {
                if self.reads.fetch_add(1, Ordering::Relaxed) == 0 {
                    return Err(Error::new(self.failure, "Synthetic expired session."));
                }
                self.wire.send(request, operation).await
            }
            _ => panic!("Device read recovery must not enumerate accounts or send controls."),
        }
    }
}

#[tokio::test]
async fn expired_device_read_logs_in_once_without_inventory_or_commands() {
    for failure in ["AUTH_REQUIRED", "MIDEA_REQUEST_FAILED"] {
        let f = Fixture::new(vec![appliances()[0].clone()]).await;
        let device = f.discover().await.remove(0);
        let record = f.store.accounts().unwrap().remove(0);
        {
            let mut secrets = f.secrets.values.lock().unwrap();
            let saved = secrets.get_mut(&record.credential_ref.to_string()).unwrap();
            let mut payload: Value = serde_json::from_slice(saved).unwrap();
            payload["version"] = json!(4);
            payload["saved_login"] = json!({
                "account":b"fictional@example.test", "password":b"PRIVATE-PASSWORD",
                "blocked":false, "last_attempt_ms":0
            });
            *saved = serde_json::to_vec(&payload).unwrap();
        }
        let wire = Arc::new(ExpiredRead {
            wire: f.wire.clone(),
            reads: AtomicUsize::new(0),
            logins: AtomicUsize::new(0),
            failure,
        });
        let adapter = MeijuAdapter::new(f.vault.clone(), wire.clone(), Arc::new(SystemClock));
        let result = adapter
            .read(&device, "main", &device.connections[0], &operation())
            .await;
        if failure == "AUTH_REQUIRED" {
            assert_eq!(result.unwrap().device_id, device.id);
            assert_eq!(wire.reads.load(Ordering::Relaxed), 2);
            assert_eq!(wire.logins.load(Ordering::Relaxed), 2);
        } else {
            assert_eq!(result.unwrap_err().code, failure);
            assert_eq!(wire.reads.load(Ordering::Relaxed), 1);
            assert_eq!(wire.logins.load(Ordering::Relaxed), 0);
        }
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
}
