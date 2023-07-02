# srs
Simple Rust Server Library inspired by Express JS

## USAGE
```rust
use srs::{Route, Server};
use srs::{Request, Response};

fn main() {
    let mut server = Server::new();
    let mut home_route = Route::new("/");
    
    home_route
        .get(handler)
        .post(|req, mut res| {
            println!("got post {:?}", req.body);
            res.status(200).send("hello post request");
        });

    server.use_route(home_route);
    // add your own routes
    server.listen("localhost:3000");
}

fn handler(req: Request, mut res: Response) {
    println!("got get {:?}", req.query);
    res.status(200).send("hello get request");
}

```
