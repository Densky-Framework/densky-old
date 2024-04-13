> This project is being rewritten to follow the actual demands.
>
> Should be found [here](https://github.com/Densky-Framework/densky)

<p align="center">
  <img src=".github/logo-720px.png" width="256px" />
  <h1 align="center"> Densky Framework </h1>
</p>

<div align="center">
  <img src="https://img.shields.io/badge/maintained-yes-blue" alt="maintained - yes">
  <a href="https://rust-lang.org/" title="Go to Rust homepage"><img src="https://img.shields.io/badge/Rust-orange?logo=rust&logoColor=white" alt="Made with Rust"></a>
  <a href="https://deno.land" title="Go to Deno homepage"><img src="https://img.shields.io/badge/Deno-blue?logo=deno&logoColor=white" alt="Made with Deno"></a>
</div>

<hr />

<details>
  <summary> <b>:memo: Table of Content</b> </summary>
  
- [:notebook: What is Densky?](#notebook-what-is-densky)
- [:star2: Features](#star2-features)
- [:rocket: Getting started](#rocket-getting-started)
- [:motorcycle: Router](#motorcycle-router)
- [Route params](#route-params)
</details>

# :notebook: What is Densky?
Densky is a backend framework based on file-routing and optimization build step.
The build step is the most important feature because this makes the app super fast without complex code. 
This expect to be deployed on the egde with [Deno Deploy](http://deploy.deno.com)

# :star2: Features
- **Deno deploy**: Support for [Deno Deploy](http://deploy.deno.com).
- [**Rust core**](packages/core): All the core code are written in Rust, this makes the app blazing fast.
- **File routing**: This solves the organization problem without imports hell.
- [**Optimized Router**](#motorcycle-router): There's an algorithm to reduce the cost of routing using simple if's and reduced loops, avoiding complex regex.
- [**Route Params**](#route-params): The route params can be used with `$` at start of filename.
- **:construction: Websockets**: Support websockets with zero config, and file routing for message topic.
- **:construction: DB ORM**: Own ORM with auto-generated endpoint specified on the model.

# :rocket: Getting started
### :construction: Work in progress (CLI is not working)
Initialize the proyect:
```bash
$ densky init
```

Run in dev mode:
```bash
$ densky dev
```

Production:
```bash
$ densky build # Build .densky, the main.ts is always there
$ densky preview
```

# :motorcycle: Router
The algorithm of the router try to make the most compact tree, here's the decision tree of algorithm:
![](https://mermaid.ink/img/pako:eNptUk2PmzAQ_SsjX7qR2IjPQDhUqoIqbau0VXcvrbh4sTdYBTuyjZIt5L93MKTJNuXCePzemw-_nlSKcZKTnab7Gp6KUgJ-GyWtVk3D9XfVWX5Xkg-MgR5jDVUtGlaSBdzfv4ci6EvyWUgG6mUCwJ0DLEpymsSKwCGHj7Rpnmn1a4BzNOtSAxUW5EcLL_MNyr8lbwVjDT9QzQe4xLcC7d-7G4kHyfhxAPe7JYoxfcP5amuc2M01QBH2T3jk7wyYGiswMHzXcmnPg4YT6Qc3CI5wMY-q5fNWhAHqilEhub4sZ-Z8UUiJkfKAODk1eQFFV8LfuqaZ23eLBquudRdvKaPuluvduKpPSkh4VraeiAZnRi4FyQ__U4ivirp-NmcMam00pzjV1UhwEKhsa2H-3WV8aWUzFh5b6YwFiiMI6x5hthTxSMt1SwVDS_YjvST4Ai0-Zo4hoxqdUcoT4mhn1eOrrEhudcc90u0ZNlQIik5uSY5GMpjlTFilt5PHndU9sqeS5D05kjxeLaN1mEVZEKySIE5Tj7xiNlovEz_IkiwK0jj2k9XJI7-VQlV_ucbEyk_jMEsSPw2d2k935yqe_gBZYwsS?type=png)
Super simplified abstract version:
- There's brother on container?
  - Expand it, and join to him
  - Join as unique son 

The result can be visualized with the following graph:
| Optimized | Others |
| --------- | ------ |
| ![Optimized Tree Graph](https://mermaid.ink/img/pako:eNqFkk1PhDAQhv8KmWQTTfhq-e7BxA0XD17Uk9ZDd1tdolDShWRXwn-3gLsCQbeXTqbP-3amnQa2kgsg8K5YuTOeUloYeq1WBroruDi8UHAovBqWdYMeZF2J2ysKqgssRuH6Er0-05sJPVh13ClOhSg1zPU2cx6MRuwancCNhS6g-BfFCyiesD817OtSKKtT9YpBg-cN4vlzOLPa_1AsPQkeFY7_6XGBGzfoTCoHE3KhcpZx_b1NZ0Ch2olcUCA65Ex9UKBFqzlWV_LxWGyBVKoWJtQlZ5VIM6anIgfyxj73Oit4Vkl1P8xLPzYmlKwA0sABiB_aXoJjL0YoDJAfRSYcddZL7MBFcRB7KPJ9NwhbE76k1K6unehE6EY-joPAjXDv9tyf9Te233o6wfU?type=png) | ![Others Tree Graph](https://mermaid.ink/img/pako:eNp9kslOwzAQhl8lGqkSSNns7D4gUeXCgQtwAnNwY7eNIHHkJlJL1HfHSWibRiW-eDT-5p_F00ImuQACG8WqrfGW0tLQBz2VXOw_KDgUPg3LekAvsqnF4x0F1RkWo3A_iy7P6OqCDiIddLJTISpNcn2NNQeJEbhEJ2ploTkOXzg85fAV-Jd611RCWV1Ijw8Bi4WBp33h6QicSdX_B90aRkePysczbd5Gx506V12ACYVQBcu5_ti206BQb0UhKBBtcqa-KNDyqDnW1PL1UGZAatUIE5qKs1qkOdP7UABZs--d9gqe11I9D5uSyXKdb3RsxUogLeyB-KHtJTj2YoTCAPlRZMJBe73EDlwUB7GHIt93g_Bowo-UWte1E-0I3cjHcRC4Ee7V3vu3PufxFxwBwfw?type=png) |

There's a significant node reduction that translate to less validations, but it have more to show, the type of validations is with a simple if. 
Here's an example of output code for simple node:
```javascript
export default function(req) {
  if (req.__accomulator__.path === "route-a") {
    // Update the accumulator for next nodes
    req.__accumulator__.segments = req.__accumulator__.segments.slice(1);
    req.__accumulator__.path = req.__accumulator__.segments.join("/");
    // ...
  }
}
```
## Route params
The route params are also supported by the same router, at the three is treated like any other node but, the compiler change the validation.
Here's an example of output code for route param node:
```javascript
const __matcher_serial = [{
  raw: "$varname",
  isVar: true,
  varname: "varname"
}];
const __matcher_start = (/* ... */) => Densky.Runtime.matcherStart(__matcher_serial, /* ... */);
export default function(req) {
  if (__matcher_start(req.__accumulator__.segments, new Map(), req.params)) {
    // Update the accumulator for next nodes
    req.__accumulator__.segments = req.__accumulator__.segments.slice(1);
    req.__accumulator__.path = req.__accumulator__.segments.join("/");
    // Now can use the param
    req.params.get("varname"); // OK
    // ...
  }
}
```
