use beacon_chain::{BeaconChain, BeaconChainTypes};
use serde::Serialize;
use slog::{info, warn};
use std::sync::Arc;
use version;

use super::{APIError, APIResult, APIService};

use hyper::{Body, Method, Request, Response, StatusCode};
use hyper_router::{Route, RouterBuilder};

#[derive(Clone)]
pub struct BeaconNodeServiceInstance<T: BeaconChainTypes + 'static> {
    pub marker: std::marker::PhantomData<T>,
}

/// A string which uniquely identifies the client implementation and its version; similar to [HTTP User-Agent](https://tools.ietf.org/html/rfc7231#section-5.5.3).
#[derive(Serialize)]
pub struct Version(String);
impl ::std::convert::From<String> for Version {
    fn from(x: String) -> Self {
        Version(x)
    }
}

/// The genesis_time configured for the beacon node, which is the unix time at which the Eth2.0 chain began.
#[derive(Serialize)]
pub struct GenesisTime(u64);
impl ::std::convert::From<u64> for GenesisTime {
    fn from(x: u64) -> Self {
        GenesisTime(x)
    }
}

impl<T: BeaconChainTypes + 'static> APIService for BeaconNodeServiceInstance<T> {
    fn add_routes(&mut self, router_builder: RouterBuilder) -> Result<RouterBuilder, hyper::Error> {
        let router_builder = router_builder
            .add(Route::get("/version").using(result_to_response!(get_version)))
            .add(Route::get("/genesis_time").using(result_to_response!(get_genesis_time::<T>)));
        Ok(router_builder)
    }
}

//TODO: Validate request stuff can be turned into a macro.
fn validate_request(req: &Request<Body>) -> Result<(), APIError> {
    let log = req.extensions().get::<slog::Logger>().unwrap();
    if req.method() != &Method::GET {
        warn!(
            log,
            "Invalid method for request to: {}",
            path_from_request!(req)
        );
        return Err(APIError::MethodNotAllowed {
            desc: "Invalid Method".to_owned(),
        });
    }
    Ok(())
}

/// Read the version string from the current Lighthouse build.
fn get_version(req: Request<Body>) -> APIResult {
    validate_request(&req)?;
    let ver = Version::from(version::version());
    let body = Body::from(
        serde_json::to_string(&ver).expect("Version should always be serialializable as JSON."),
    );
    Ok(success_response!(body))
}

/// Read the genesis time from the current beacon chain state.
fn get_genesis_time<T: BeaconChainTypes + 'static>(req: Request<Body>) -> APIResult {
    validate_request(&req)?;
    let beacon_chain = req.extensions().get::<Arc<BeaconChain<T>>>().unwrap();
    let gen_time = {
        let state = beacon_chain.current_state();
        state.genesis_time
    };
    let body = Body::from(
        serde_json::to_string(&gen_time)
            .expect("Genesis should time always have a valid JSON serialization."),
    );
    Ok(success_response!(body))
}