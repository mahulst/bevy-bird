# BEVY Bird

## build for web:
```bash
$ wasm-pack build --target web --release --no-default-features
$ python -m SimpleHTTPServer 8000 
```
Navigate to http://localhost:8000

## Ideas
1. refactor code into modules
2. move player instead of obstacles (make camera follow player)
3. Add github actions & pages build for web version
4. actual 3d models
5. simple 3d animation
6. high scores
7. ios export
8. spinning 3d models instead of static

## Questions:
1. positions collider vs node vs pbr bundle
Always use rigid body position collider will attach to rigidbody and transform.
2. Render pipelines -> How to sort draw order for different components -> best practices -> documentation
3. Ui, multiple systems to add -> positioning row vs column

