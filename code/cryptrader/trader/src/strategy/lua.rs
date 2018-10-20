use ::prelude::*;

use std::error::Error;
use common::rlua::{self, Lua, LightUserData, FromLua, ToLua, UserData, UserDataMethods};


pub struct LuaStrategy {
    lua: Box<Lua>,
}


impl LuaStrategy {
    pub fn from_file(path: &str) -> Result<LuaStrategy> {
        return Self::new(&::std::fs::read_string(::std::path::Path::new(path))?);
    }
    pub fn new(src: &str) -> Result<LuaStrategy> {
        let mut lua = Box::new(Lua::new());

        register_ta(&mut lua)?;
        disable_io(&mut lua)?;
        {
            let fun = lua.load(src, "Strategy".into())?;

            lua.globals().set("__strategy", fun)?;
        }

        return Ok(LuaStrategy {
            lua
        });
    }
}

use ::strategy::*;

impl ::strategy::TradingStrategy for LuaStrategy {
    fn decide(&self, data: &StrategyInput) -> TradingDecision {
        self.lua.globals().set("__ohlc",
                               data.candles
                                   .iter()
                                   .map(|(k, v)| { LuaOhlc(v.clone()) })
                                   .collect::<Vec<LuaOhlc>>()).unwrap();

        let strat: rlua::Function = self.lua.globals().get("__strategy").unwrap();

        let res = strat.call::<_, f64>(()).unwrap();
        // error!("Eval Res : {:?}", res);
        if res < 0.0 {
            return TradingDecision::Short;
        } else if res > 0.0 {
            return TradingDecision::Long;
        } else {
            return TradingDecision::Indeterminate;
        }
    }
}

#[derive(Clone, Debug)]
struct LuaOhlc(pub Ohlc);


impl UserData for LuaOhlc {
    fn add_methods(_methods: &mut UserDataMethods<Self>) {
        _methods.add_method("time", |_, ohlc, ()| Ok(ohlc.0.time));
        _methods.add_method("open", |_, ohlc, ()| Ok(ohlc.0.open));
        _methods.add_method("high", |_, ohlc, ()| Ok(ohlc.0.high));
        _methods.add_method("low", |_, ohlc, ()| Ok(ohlc.0.low));
        _methods.add_method("close", |_, ohlc, ()| Ok(ohlc.0.close));
    }
}

pub struct LuaIndicator<T: ta::Next<f64> + 'static + ta::Reset> {
    indicator: T,
}

impl<T> UserData for LuaIndicator<T>
    where T: ta::Next<f64> + 'static + ta::Reset,
          for<'aa> T::Output: rlua::ToLuaMulti<'aa> + Clone + 'static
{
    fn add_methods(_methods: &mut UserDataMethods<Self>) {
        _methods.add_meta_method_mut(rlua::MetaMethod::Call, |lua, this, ()| {
            let ohlc = lua.globals().get::<&str, Vec<LuaOhlc>>("__ohlc").unwrap();
            this.indicator.reset();

            unsafe {
                let mut last = mem::zeroed();

                for x in ohlc.iter() {
                    last = this.indicator.next(x.0.close);
                }
                return Ok(last);
            }
        });
    }
}

pub struct LuaPairData {}

impl rlua::UserData for LuaPairData {
    fn add_methods(_methods: &mut UserDataMethods<Self>) {}
}

fn disable_io(lua: &mut Lua) -> Result<()> {
    let src = r#"
local oldprint = print
print = function(...)
    oldprint("In ur print!");
    oldprint(...);
end"#;

    lua.exec::<()>(src, None)?;

    Ok(())
}

fn register_ta(lua: &mut Lua) -> Result<rlua::Table> {
    let mut ta = lua.create_table()?;
    ta.set("ema", lua.create_function(|lua, (period, ): (u32, )| {
        Ok(LuaIndicator {
            indicator: ExponentialMovingAverage::new(period).unwrap()
        })
    })?)?;

    ta.set("sma", lua.create_function(|lua, (period, ): (u32, )| {
        Ok(LuaIndicator {
            indicator: SimpleMovingAverage::new(period).unwrap()
        })
    })?)?;

    ta.set("macd", lua.create_function(|lua, (fast, slow, sig): (u32, u32, u32)| {
        Ok(LuaIndicator {
            indicator: MovingAverageConvergenceDivergence::new(fast, slow, sig).unwrap()
        })
    })?)?;
    ta.set("rsi", lua.create_function(|lua, period: u32| {
        Ok(LuaIndicator {
            indicator: ta::indicators::RelativeStrengthIndex::new(period).unwrap()
        })
    })?)?;

    let tr = lua.create_function(|lua, ()| {
        Ok(LuaIndicator {
            indicator: ta::indicators::TrueRange::new()
        })
    })?;
    ta.set("tr", tr)?;

    let atr = lua.create_function(|lua, period: u32| {
        Ok(LuaIndicator {
            indicator: ta::indicators::AverageTrueRange::new(period).unwrap()
        })
    })?;
    ta.set("atr", atr)?;

    let max = lua.create_function(|lua, period: u32| {
        Ok(LuaIndicator {
            indicator: ta::indicators::Maximum::new(period).unwrap()
        })
    })?;
    ta.set("max", max)?;

    let min = lua.create_function(|lua, period: u32| {
        Ok(LuaIndicator {
            indicator: ta::indicators::Minimum::new(period).unwrap()
        })
    })?;
    ta.set("min", min)?;

    let ss = lua.create_function(|lua, (a, b): (u32, u32)| {
        Ok(LuaIndicator {
            indicator: ta::indicators::SlowStochastic::new(a, b).unwrap()
        })
    })?;

    ta.set("ss", ss)?;

    let fs = lua.create_function(|lua, period: u32| {
        Ok(LuaIndicator {
            indicator: ta::indicators::FastStochastic::new(period).unwrap()
        })
    })?;
    ta.set("fs", fs)?;

    lua.globals().set("ta", ta.clone())?;
    Ok(ta)
}
