use crate::prelude::*;

use crate::actix_arch::balancing::*;
use actix_arch::balancing::WorkerRequest;
pub use strat_eval::EvalError;
use futures_util::FutureExt;
use actix_arch::balancing::WorkerProxy;
use actix::msgs::StopArbiter;


#[derive(Debug, Serialize, Deserialize)]
pub struct EvalRequest {
    pub strat_id : i32,
    pub spec : OhlcSpec,
    pub last: i64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EvalResponse {
    pub decision : TradingDecision,
    pub spec : OhlcSpec,
}

#[derive(Debug)]
pub struct EvalService;

impl ServiceInfo for EvalService {
    type RequestType = EvalRequest;
    type ResponseType = Result<EvalResponse,EvalError>;
    const ENDPOINT: &'static str = "actix://ingest:42044/eval";
}


pub struct EvalWorker {
    db: Database,
    proxy : Option<Addr<WorkerProxy<EvalService>>>,
}

impl Actor for EvalWorker { type Context = Context<Self>;

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        if let Some(proxy) = self.proxy.clone() {
            // TODO: Send stop message
        }
        return Running::Stop;
    }
}

impl EvalWorker {
    pub fn new(handle: ContextHandle, db: Database) -> Addr<Self> {
        Actor::create(|ctx| {
            Self::init(ctx,handle,db)
        })
    }

    pub fn init(ctx : &mut Context<Self>, handle : ContextHandle, db : Database) -> Self {
        ctx.spawn(wrap_future(WorkerProxy::new(handle.clone(), ctx.address().recipient()))
            .then(|res,mut this : &mut Self,ctx| {
                this.proxy = Some(res.unwrap());
                afut::ok(())
            })
        );
        Self {
            db,
            proxy: None,
        }
    }

}

impl Handler<ServiceRequest<EvalService>> for EvalWorker {
    type Result = Response<Result<EvalResponse,EvalError>, RemoteError>;

    fn handle(&mut self, msg: ServiceRequest<EvalService>, ctx: &mut Self::Context) -> Self::Result {
        let req : EvalRequest = msg.0;

        let strat = self.db.strategy_data(req.strat_id);
        let data = self.db.resampled_ohlc_values(req.spec.clone(),req.last - (60 * 60 * 12));

        let fut = Future::join(strat,data);
        let fut = Future::map(fut ,             |((strat,user),data)| {
                let data = data.into_iter().map(|x| (x.time,x)).collect();
                let res = strat_eval::eval(data,strat.body)?;
                let res = EvalResponse {
                    spec : req.spec,
                    decision : res
                };
                Ok(res)
            });

        return Response::r#async(fut.drop_err().set_err(RemoteError::Timeout));
    }
}

