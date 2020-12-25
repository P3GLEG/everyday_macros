# everyday_macros

This crate contains all the procedural macros you might use in your everyday life while coding. Things like a thread sleep timer or a retry on error. 


## 


## Getting Started

To a thread sleep timer to the prologue of your function. It is uses `std::thread::sleep` for default functions and `tokio::time::sleep` for async. It also has the ability to add a jitter with a range of [0, N).
```rust
	#[wait_for(seconds=3)]
	fn my_func_to_sleep(args: any_amount){
		...
	}

	#[wait_for(seconds=3, jitter)]
	fn my_func_to_sleep(args: any_amount){
		...
	}
```


To add a harness around your function add the follow above it. Currently does not work on async!
```rust
	#[retry(times=3)]
	fn my_func_that_can_fail(args: any_amount) -> Result<(), Err>{
		...
	}
```


## Running the tests

Just run 
```bash
git clone https://github.com/P3GLEG/everyday_macros && cd everyday_macros
cargo test -- --color always --nocapture
``` 


## Authors

* **Paul Ganea** - *Initial work* - [pegleg](https://github.com/p3gleg)

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details
