# BEVY Bir
Flappy bird clone in bevy

![alt text][screenshot]

[screenshot]: https://raw.githubusercontent.com/mahulst/bevy-bird/master/docs/example.gif "Example gif"

## build for web:
```bash
$ wasm-pack build --target web --release --no-default-features
$ python -m SimpleHTTPServer 8000 
```
Navigate to http://localhost:8000

## Ideas
2. move player instead of obstacles (make camera follow player)
4. Add github actions & pages build for web version
5. actual 3d models
3. refactor code systems into plugins
6. simple 3d animation
7. high scores
8. ios export
9. spinning 3d models instead of static

## Questions:
1. positions collider vs node vs pbr bundle
Always use rigid body position collider will attach to rigidbody and transform.
2. Render pipelines -> How to sort draw order for different components -> best practices -> documentation
3. Ui, multiple systems to add -> positioning row vs column
4. What's up with the double declartion of modules in main & lib
