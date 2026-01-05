# tiles design doc

- each tile should contain at most one item and at most one blocking object
- this can be different depending on the type of tile, but serves as a hard cap
    - ex. wall tiles should not permit items to lie on top of them

- tiles should contain a "default" renderable that renders when nothing is there
- tiles should contain object ids of item / blocking obj. located there
