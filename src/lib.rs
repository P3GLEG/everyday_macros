#![crate_name = "everyday_macros"]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    AttributeArgs,
    Block,
    Error as SynError,
    Ident,
    ItemFn,
    Lit,
    LitInt,
    Meta,
    MetaNameValue,
    NestedMeta,
    Path,
    Result as SynResult,
    ReturnType,
    Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Simple struct to parse `Attribute` with name=value
struct KeyValue {
    key: Ident,
    value: LitInt,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let key = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(KeyValue { key, value })
    }
}

enum Value {
    Int(u64),
    Float(f64),
}

struct Config {
    seconds: Value,
    jitter: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            seconds: Value::Int(0),
            jitter: false,
        }
    }
}

fn parse_namevalue(namevalue: MetaNameValue, config: &mut Config) -> Result<(), SynError> {
    let ident = namevalue.path.get_ident();
    if ident.is_none() {
        let msg = "Must have specified ident";
        return Err(SynError::new_spanned(ident, msg));
    }
    let name = ident.unwrap().to_string().to_lowercase();
    match name.as_str() {
        "seconds" => match namevalue.lit {
            Lit::Int(i) => {
                let i = i.base10_parse::<u64>().unwrap();
                config.seconds = Value::Int(i);
            }
            Lit::Float(i) => {
                let i = i.base10_parse::<f64>().unwrap();
                config.seconds = Value::Float(i);
            }
            _ => {
                let msg = "You must provide a int value!";
                return Err(SynError::new_spanned(namevalue.lit, msg));
            }
        },
        "jitter" => match namevalue.lit {
            Lit::Bool(b) => {
                config.jitter = b.value;
            }
            _ => {
                let msg = "You must provide a boolean value!";
                return Err(SynError::new_spanned(namevalue.lit, msg));
            }
        },

        name => {
            let msg = format!(
                "Unknown attribute {} is specified; expected one of: `jitter`, `seconds`",
                name
            );
            return Err(SynError::new_spanned(name, msg));
        }
    }
    Ok(())
}

fn parse_path(path: Path, config: &mut Config) -> Result<(), SynError> {
    let ident = path.get_ident();
    if ident.is_none() {
        let msg = "Must have specified ident";
        return Err(SynError::new_spanned(ident, msg));
    }
    let name = ident.unwrap().to_string().to_lowercase();
    match name.as_str() {
        "jitter" => {
            config.jitter = true;
        }
        _ => {
            let msg = format!("Unable to understand the ident {}", name);
            return Err(SynError::new_spanned(name, msg));
        }
    }
    Ok(())
}

fn parse_args(args: AttributeArgs) -> Result<Config, SynError> {
    let mut config = Config::default();
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(namevalue)) => {
                if let Err(e) = parse_namevalue(namevalue, &mut config) {
                    return Err(e);
                }
            }
            NestedMeta::Meta(Meta::Path(path)) => {
                if let Err(e) = parse_path(path, &mut config) {
                    return Err(e);
                }
            }
            _ => {
                let msg = format!("Unable to parse {:#?}", arg);
                return Err(SynError::new_spanned(arg, msg));
            }
        }
    }
    Ok(config)
}

fn get_sleep_duration(config: Config) -> proc_macro2::TokenStream {
    match config.seconds {
        Value::Int(i) => {
            if config.jitter {
                quote! {
                    let mut rng = rand::thread_rng();
                    let dur = std::time::Duration::from_secs(rng.gen_range(0, #i));
                }
            } else {
                quote! {
                    let dur = std::time::Duration::from_secs(#i);
                }
            }
        }
        Value::Float(f) => {
            if config.jitter {
                quote! {
                    let mut rng = rand::thread_rng();
                    let dur = std::time::Duration::from_secs_f64(rng.gen_range(0.0, #f));
                }
            } else {
                quote! {
                    let dur = std::time::Duration::from_secs_f64(#f);
                }
            }
        }
    }
}

fn jittered(config: Config, is_async: bool) -> proc_macro2::TokenStream {
    let sleep_dur = get_sleep_duration(config);
    if is_async {
        quote! {
            #sleep_dur
            tokio::time::sleep(dur).await;
        }
    } else {
        quote! {
            #sleep_dur
            std::thread::sleep(dur);
        }
    }
}

/// To add a sleep timer to the beginning of each function call write the following proc_macro
/// `#[everyday_macro::wait_for(seconds = 10, jitter)]`
/// You can also specify floats and turn jitter on or off. jitter is off by default.
/// `#[everyday_macro::wait_for(seconds = 3.5, jitter = true)]`
#[proc_macro_attribute]
pub fn wait_for(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let res = parse_args(args);
    let config;
    match res {
        Ok(conf) => {
            config = conf;
        }
        Err(e) => return e.to_compile_error().into(),
    }
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    let Block {
        brace_token: _,
        stmts,
    } = *block;
    let jit = jittered(config, sig.asyncness.is_some());
    let func = match sig.asyncness.is_some() {
        true => {
            quote! {
                #(#attrs)*
                #vis #sig {
                    #jit
                    #(#stmts)*
                }
            }
        }
        false => {
            quote! {
                #(#attrs)*
                #vis #sig {
                    #jit
                    #(#stmts)*
                }
            }
        }
    };
    func.into()
}

/// To wrap a function around a retry, specify how many times you should invoke the function
/// before quitting. It currently only supports non-async functions!
/// `#[everyday_macro::retry(times=3)]`
#[proc_macro_attribute]
pub fn retry(args: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    if sig.asyncness.is_some() {
        return SynError::new_spanned(sig, "Unable to retry async funcs yet!")
            .to_compile_error()
            .into();
    }
    if sig.output == ReturnType::Default {
        return SynError::new_spanned(sig, "You must specify a function with a Result!")
            .to_compile_error()
            .into();
    }
    let KeyValue { key, value } = parse_macro_input!(args as KeyValue);
    if key != "times" {
        return SynError::new_spanned(key, "Unable to understand argument!")
            .to_compile_error()
            .into();
    }
    let n = value
        .base10_parse::<u64>()
        .expect("Should be a valid u64 value!");
    let wrapped_func = quote! {
        #(#attrs)*
        #vis #sig {
            let mut tries: u64 = #n;
        let mut func = || { #block };
            let res = loop {
                let res = func();
                if res.is_ok(){
                   break res;
                }
                if tries > 0{
                    tries -=1;
                    continue;
                }
                break res;
            };
        res
        }
    };
    wrapped_func.into()
}
