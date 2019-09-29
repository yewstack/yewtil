# Yewtil
Utility crate for the Yew web framework.

## Purpose
Provide a place for commonly used utilities for Yew to reside without them having to be included in the Yew crate itself.
As a consequence of this, the Yew crate is make changes that may cause breakages in this crate.

## Features
Currently this crate supports two features.
* `neq_assign` makes assigning props and returning a relevant ShouldRender value easier.
* Pure Components. Implement pure components using two traits: `PureComponent` and `Emissive`, the latter of which can be derived in most cases.


## Demo
Check out the [demo example](https://github.com/hgzimmerman/yewtil/examples/demo/) to see how Pure Components work.