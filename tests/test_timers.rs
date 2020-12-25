use everyday_macros::retry;
use everyday_macros::wait_for;
use std::time::Instant;

#[retry(times = 3)]
fn retry_3(i: &mut i32) -> Result<(), ()> {
    if *i==3 {
        Ok(())
    } else {
        *i+=1;
        Err(())
    }
}

#[retry(times = 100)]
fn retry_100(i: &mut i32) -> Result<(), ()> {
    if *i==100 {
        Ok(())
    } else {
        *i+=1;
        Err(())
    }
}

#[test]
fn test_retry(){
    assert_eq!(retry_3(&mut -1).is_ok(), false);
    assert_eq!(retry_3(&mut 0).is_ok(), true);
    let mut should_be_100 = 0;
    retry_100(&mut should_be_100).unwrap();
    assert_eq!(should_be_100, 100);
}

#[wait_for(seconds = 3)]
fn tester() -> Instant {
    std::time::Instant::now()
}

#[wait_for(seconds = 3)]
async fn async_tester() -> Instant {
    std::time::Instant::now()
}

#[test]
fn test_std_sleep() {
    let now = std::time::Instant::now();
    let one = tester();
    assert_eq!(now.elapsed().as_secs(), 3);
    tester();
    assert_eq!(one.elapsed().as_secs(), 3);
}

#[tokio::test]
async fn test_async_tokio_sleep() {
    let now = std::time::Instant::now();
    let one = async_tester().await;
    assert_eq!(now.elapsed().as_secs(), 3);
    async_tester().await;
    assert_eq!(one.elapsed().as_secs(), 3);
}
