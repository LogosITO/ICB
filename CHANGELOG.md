# Changelog

## [0.4.0](https://github.com/LogosITO/ICB/compare/v0.3.0...v0.4.0) (2026-05-04)


### Features

* upload ZIP, CORS fix, robust directory traversal, auto-detect language ([135d5a0](https://github.com/LogosITO/ICB/commit/135d5a014b139b6d6b948ee81e8ff209ebcc944d))

## [0.3.0](https://github.com/LogosITO/ICB/compare/v0.2.0...v0.3.0) (2026-05-04)


### Features

* **benches:** Expanded benchmarks with go and ruby supported languages ([c996221](https://github.com/LogosITO/ICB/commit/c9962214b91eab7da7dd5cfc50630b273e7162c3))

## [0.2.0](https://github.com/LogosITO/ICB/compare/v0.1.0...v0.2.0) (2026-05-04)


### Features

* add tree-sitter-go and tree-sitter-ruby parsers ([a1b20b1](https://github.com/LogosITO/ICB/commit/a1b20b1f21a154a472a4dd8479769db61c9826e5))

## 0.1.0 (2026-05-04)


### Features

* add analytics API, tests, and modular server structure ([824da9d](https://github.com/LogosITO/ICB/commit/824da9de1f860d81829821cb61c9ad486a72a89a))
* add benchmark dashboards, tree-sitter-cpp benchmarks ([d72c5f6](https://github.com/LogosITO/ICB/commit/d72c5f626f39c6aa95b2c4c28d48455cf6f4e669))
* add diff endpoint and viewer, fix graph height, multi‑edge support ([e8a9b09](https://github.com/LogosITO/ICB/commit/e8a9b09126bb39eb0d3894b509efceb502ff1000))
* add persistent graph cache with bincode+zstd ([77e6aae](https://github.com/LogosITO/ICB/commit/77e6aaeadde50de9831d336a507bcedd44b37f46))
* Add Python demo project for multi-file testing ([c51e07d](https://github.com/LogosITO/ICB/commit/c51e07dbe1ff056637d4771f5e9d95ea19a1ad97))
* add tree-sitter-cpp parser with benchmarks ([3270a16](https://github.com/LogosITO/ICB/commit/3270a1611d069edbadd03540cdfbb6b2b1c0c745))
* add working Clang C/C++ frontend with tempfile and tests ([08577a3](https://github.com/LogosITO/ICB/commit/08577a39c4a3b6eb71554eafadb0eed512319afe))
* Added audit and benchmarks github actions for visibility and stability ([9c6b635](https://github.com/LogosITO/ICB/commit/9c6b6357f8d8f10ceef74cd6832183bdc8cab590))
* Added CodeQL ([eeb3522](https://github.com/LogosITO/ICB/commit/eeb35225faa301c77787f994d4358800e90fb907))
* Added first implementation of new easy-to-use report system ([fc28dd3](https://github.com/LogosITO/ICB/commit/fc28dd330e29ed63717570c214ca312ddd7d43e5))
* Added first web part ([216f3f5](https://github.com/LogosITO/ICB/commit/216f3f54091e8ac5617b5bf9c3bdfa11a3ced9a6))
* Added icb-server implementation ([074f8a0](https://github.com/LogosITO/ICB/commit/074f8a00a19921d3accaa38454975908c26dd402))
* Autodocs ([f600b9d](https://github.com/LogosITO/ICB/commit/f600b9d910f137a0c8f2b40a5361df1a084ecbbc))
* First parser ([e0c7aab](https://github.com/LogosITO/ICB/commit/e0c7aab017d152af7dbe840697bf0089786fed67))
* full C++ support with compile_commands.json, parallel parsing, and CLI integration ([24992c1](https://github.com/LogosITO/ICB/commit/24992c1fd1a509dc26444bf0d12dee1798a9837e))
* high-performance C++ parser, graph benchmarks, UI fixes ([9f735e2](https://github.com/LogosITO/ICB/commit/9f735e2326783ee596cd073491f16a76489399eb))
* **icb-clang:** high‑performance parser with comprehensive benchmarks ([ebf87ed](https://github.com/LogosITO/ICB/commit/ebf87ed7ca0083dd642a2fefdda4d7074baf700f))
* Initial ICB structure with working parser, core and CLI ([0c3ac36](https://github.com/LogosITO/ICB/commit/0c3ac360372c16790139548c261ed1279d923870))
* new scalable architecture with parser, graph core, and CLI ([1788a76](https://github.com/LogosITO/ICB/commit/1788a768ea31985b5a33cf2cbf99df4f25c4962f))
* parallel indexing, call graph queries, and DOT export ([d48a757](https://github.com/LogosITO/ICB/commit/d48a757b86e7e76f126225369225c6ac181627ee))
* Python support in server, fix Sigma import, drop icb-clang dependency ([c353613](https://github.com/LogosITO/ICB/commit/c353613b30823df552ac12a230aad704b7fdda0b))
* rebuild architecture with docs and tests ([3f12405](https://github.com/LogosITO/ICB/commit/3f124054b1d08740a04c0fb4959dbad93edb08b7))
* universal heuristic parser for 98% of languages ([f883fda](https://github.com/LogosITO/ICB/commit/f883fda07cecee3efffb63aabe9e60f3c57ff060))
* Updated frontend ([d74b412](https://github.com/LogosITO/ICB/commit/d74b412d54214be7ce483755836786042af7ab86))
* Upgraded web implementation of graph ([3346b08](https://github.com/LogosITO/ICB/commit/3346b08cd8745c43fa6a3bc27a032a0489fd0116))
* working analytics dashboard + graph fixes + monochrome UI ([721d272](https://github.com/LogosITO/ICB/commit/721d272b40475e20cbfe91247b4454ec02611aae))
* working analytics server + dashboard foundation ([0f307f5](https://github.com/LogosITO/ICB/commit/0f307f5ad24913addf625662c3df96fe51901b56))


### Bug Fixes

* Added --exclude icb-clang in docs ([6bb586e](https://github.com/LogosITO/ICB/commit/6bb586e4b89c69cc8d8fb817b3f39101bc23dc28))
* Changed parse_language function for supporting for all languages by heurestic parser ([19bf390](https://github.com/LogosITO/ICB/commit/19bf390d2145c8e0cb62868d94d2e7434de349fa))
* **ci:** Added clang install in ci ([aeb9fc1](https://github.com/LogosITO/ICB/commit/aeb9fc18bb08fca8ed849fac4208ae7033f7da85))
* **ci:** Changed github secret token to custom fine-grained token ([f0fd304](https://github.com/LogosITO/ICB/commit/f0fd30499deff3abf714419e4b7605e41ad499e9))
* **ci:** Fixed Cargo.toml for new release-please github action ([3abc2cc](https://github.com/LogosITO/ICB/commit/3abc2cc5e488ff3ff5f7203b0db6233e7924cebb))
* **ci:** Fixed problem in ci with CLand/LLVM installing ([51721e0](https://github.com/LogosITO/ICB/commit/51721e037f8be016309a2eb141edfdac0c38d70f))
* **ci:** Hotfix ([21b12bd](https://github.com/LogosITO/ICB/commit/21b12bd6f9b061c2d464ddc468db4845d0b231e9))
* **ci:** Hotfix ([c8927e3](https://github.com/LogosITO/ICB/commit/c8927e365020b48e4dd22f563452c04d2542b103))
* correct deny.toml for cargo-deny ([8d52221](https://github.com/LogosITO/ICB/commit/8d52221a484cc9b222448d147f2d470d1bbb0804))
* **docs:** CodeQL vulnerability fix ([07132e5](https://github.com/LogosITO/ICB/commit/07132e5cc09e51fe8c44e9b0c4af60393f28aeb9))
* Fix --no-system-headers flag ([440f513](https://github.com/LogosITO/ICB/commit/440f513bf1e0bb63f91face595e03a73fff569dd))
* Fixed benchmark starting scenaries ([ddf6fd5](https://github.com/LogosITO/ICB/commit/ddf6fd59e02d963a958243730a1b00f47dad80a0))
* Fixed builder.rs file ([fee5338](https://github.com/LogosITO/ICB/commit/fee53388b6db3ae2924b13f66fad3ec2d31d6507))
* Fixed icb-lang/src/lib.rs ([3624632](https://github.com/LogosITO/ICB/commit/362463268042f5e4b6052a38f5298c5be1bd4017))
* Heurestic parser algorythm changed from tokenizator to token counter + regex ([bc48094](https://github.com/LogosITO/ICB/commit/bc48094ac36df7a7afc831186ec03b7a37aa56b6))
* Heuristic_parser.rs fixed for go test project ([2cd78ea](https://github.com/LogosITO/ICB/commit/2cd78ea5bf9b434be26698d4109de77c9f9b1e1c))
* Hotfix ([5a227ec](https://github.com/LogosITO/ICB/commit/5a227ec5e96e59db9d64fbc7101da41d3e6ebeb4))
* Hotfix ([2c8654e](https://github.com/LogosITO/ICB/commit/2c8654ea5ff850a592acd968427a9f5743ac1246))
* Hotfix ([38c2422](https://github.com/LogosITO/ICB/commit/38c2422b680be7daaea18c877cd2396ec334f954))
* Hotfix ([b181224](https://github.com/LogosITO/ICB/commit/b1812246fdcda30a3405b80056a53ff9ea3e93be))
* Hotfix ([8ff6393](https://github.com/LogosITO/ICB/commit/8ff6393783f78d46c8a2ffae075f140bd4484e5c))
* Hotfix ([ac428fd](https://github.com/LogosITO/ICB/commit/ac428fd17518080abbf765dea79744034f9cb1fe))
* Hotfix ([e575cd6](https://github.com/LogosITO/ICB/commit/e575cd6cf1895175c00a87bbb3ff134aeac4312c))
* Hotfix 2 ([b90d9a4](https://github.com/LogosITO/ICB/commit/b90d9a469f6123979a9be9fb41f03d775e1df0ec))
* Hotfix 2 ([139d6f7](https://github.com/LogosITO/ICB/commit/139d6f75ba9834bc9a0c666619790550fd9ade45))
* Hotfix 3 ([5ee6264](https://github.com/LogosITO/ICB/commit/5ee6264c542665269e61f241a2b2a29732734a4d))
* Hotfix 3 ([07586e2](https://github.com/LogosITO/ICB/commit/07586e2db4d5ecab2f23bc7a43d46f4661070095))
* Hotfix 4 ([d8e9c87](https://github.com/LogosITO/ICB/commit/d8e9c87fa3630429129878fabea06decf15aea55))
* Hotfix2 ([07507f5](https://github.com/LogosITO/ICB/commit/07507f5f8ff9aebed5b0dbbc070c940383e8b7ab))
* **icb-server:** convert Clang USR to readable names in graph ([8794334](https://github.com/LogosITO/ICB/commit/879433462dbbba55cd63dcd1c8f7055f0c59ba5d))
* Install clang fix ([71b0bce](https://github.com/LogosITO/ICB/commit/71b0bce4914959a78515694139878cfb415abdf5))
* prevent infinite loop in detect_complex_functions, add recursion limit ([a3f0ab8](https://github.com/LogosITO/ICB/commit/a3f0ab832371af46fe5955e0359b4bba78f4edbf))
* prevent infinite loop in detect_complex_functions, add recursion limit ([7108e8e](https://github.com/LogosITO/ICB/commit/7108e8ec32e723704df7be8b53f9adf9744fb5a0))
* Removed audit.yml bc i cant setup it ([d9330a5](https://github.com/LogosITO/ICB/commit/d9330a5a500cff3e098fd5747abee64546e8bdfc))
* Tests fixed in lang/python.rs ([8fe9ca7](https://github.com/LogosITO/ICB/commit/8fe9ca721c93fcba116689fd71c66422c6000993))
* update dead code test to reflect unreachable self‑loop function ([8f897d5](https://github.com/LogosITO/ICB/commit/8f897d5c4317a5a96d81a01595b6fd2a391170d7))


### Performance Improvements

* **icb-clang:** optimize parser, add benchmarks, fix graph names ([33b9b52](https://github.com/LogosITO/ICB/commit/33b9b529e647ca27bbd0979e658361879978a7a3))

��#   C h a n g e l o g  
 
