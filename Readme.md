# Yewtil
Utility crate for the Yew web framework.

## Purpose
Provide a place for commonly used utilities for Yew to reside without them having to be included in the Yew crate itself.
As a consequence of this, the Yew crate is free to make changes that may cause breakages in this crate.

## Features
Currently this crate supports two features.
* `neq_assign` - makes assigning props and returning a relevant ShouldRender value easier.
* Pure Components - implement pure components using two traits: `PureComponent` and `Emissive`, the latter of which can be derived in most cases. 
This should make it much easier to define simple Components that don't hold state.

## Upcoming Features
* Fetch Monad or Component - a datatype that holds either a fetch task, nothing, a network/serialization error, or a resolved data type and is able to render itself.

## Demo
Check out the [demo example](https://github.com/hgzimmerman/yewtil/tree/master/examples/demo) to see how Pure Components work.


## Update Schedule
This crate will target stable Yew.

As new idioms are introduced to Yew, this crate may see updates, but given the rarity of those, this crate may sit unaltered for some time.

Minor bugfixes or Yew breakage fixes will result in a minor version bump.
Additional features will increment the patch version.
This crate will reach 1.0.0 when Yew itself does.

This crate aims to be more permissive in what is allowed in than Yew, so if you have a function or trait you would like to share, please open a PR or Issue.
