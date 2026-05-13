# Changelog

## [0.10.7](https://github.com/LogosITO/ICB/compare/v0.10.6...v0.10.7) (2026-05-13)


### Bug Fixes

* **ci:** Benchmarks and perf github actions another try to fix ([84fd197](https://github.com/LogosITO/ICB/commit/84fd197e2bb0a81b7b305204efc8b30ed0e24775))

## [0.10.6](https://github.com/LogosITO/ICB/compare/v0.10.5...v0.10.6) (2026-05-10)


### Bug Fixes

* add LLVM/Clang dependencies and upgrade to 8-core runner for benchmarks ([f58e1e8](https://github.com/LogosITO/ICB/commit/f58e1e8dcd269bf13a82cb4ba4e7dc16d5cd16d2))
* install libclang dependencies for benchmark workflow ([9b64ae1](https://github.com/LogosITO/ICB/commit/9b64ae1b58a831d29b2b3e6515aadc7d427c0162))

## [0.10.5](https://github.com/LogosITO/ICB/compare/v0.10.4...v0.10.5) (2026-05-10)


### Bug Fixes

* **benches:** safe and production-ready CI ([33fcbda](https://github.com/LogosITO/ICB/commit/33fcbda74b75d23e5423a3e9799ce8afda9b89b6))

## [0.10.4](https://github.com/LogosITO/ICB/compare/v0.10.3...v0.10.4) (2026-05-10)


### Bug Fixes

* **benches:** another try ([7f7d79c](https://github.com/LogosITO/ICB/commit/7f7d79c617c246d61367ab3723e177eec1c4bad5))

## [0.10.3](https://github.com/LogosITO/ICB/compare/v0.10.2...v0.10.3) (2026-05-10)


### Bug Fixes

* **benches:** generate-bench-json script ([896f122](https://github.com/LogosITO/ICB/commit/896f1227496bb7aea850b418e3fd327d92d2634c))

## [0.10.2](https://github.com/LogosITO/ICB/compare/v0.10.1...v0.10.2) (2026-05-10)


### Bug Fixes

* **bench:** unify benchmark system across all crates ([6615a8c](https://github.com/LogosITO/ICB/commit/6615a8c99982e5a43f2e0758e1ddeee0060ff2f7))

## [0.10.1](https://github.com/LogosITO/ICB/compare/v0.10.0...v0.10.1) (2026-05-10)


### Bug Fixes

* **ci:** release-please stable ver ([e766c27](https://github.com/LogosITO/ICB/commit/e766c27555832a47405f557176502f51d8d9d0d2))

## [0.10.0](https://github.com/LogosITO/ICB/compare/v0.9.0...v0.10.0) (2026-05-10)


### Features

* add analytics API, tests, and modular server structure ([824da9d](https://github.com/LogosITO/ICB/commit/824da9de1f860d81829821cb61c9ad486a72a89a))
* add benchmark dashboards, tree-sitter-cpp benchmarks ([d72c5f6](https://github.com/LogosITO/ICB/commit/d72c5f626f39c6aa95b2c4c28d48455cf6f4e669))
* add diff endpoint and viewer, fix graph height, multi‑edge support ([e8a9b09](https://github.com/LogosITO/ICB/commit/e8a9b09126bb39eb0d3894b509efceb502ff1000))
* add persistent graph cache with bincode+zstd ([77e6aae](https://github.com/LogosITO/ICB/commit/77e6aaeadde50de9831d336a507bcedd44b37f46))
* Add Python demo project for multi-file testing ([c51e07d](https://github.com/LogosITO/ICB/commit/c51e07dbe1ff056637d4771f5e9d95ea19a1ad97))
* add tree visualization, removing graph explosion fallback and adding safety limits ([a275ddc](https://github.com/LogosITO/ICB/commit/a275ddcd13164434ecdff9213178c3025d974a80))
* add tree-sitter-cpp parser with benchmarks ([3270a16](https://github.com/LogosITO/ICB/commit/3270a1611d069edbadd03540cdfbb6b2b1c0c745))
* add tree-sitter-go and tree-sitter-ruby parsers ([a1b20b1](https://github.com/LogosITO/ICB/commit/a1b20b1f21a154a472a4dd8479769db61c9826e5))
* add working Clang C/C++ frontend with tempfile and tests ([08577a3](https://github.com/LogosITO/ICB/commit/08577a39c4a3b6eb71554eafadb0eed512319afe))
* Added audit and benchmarks github actions for visibility and stability ([9c6b635](https://github.com/LogosITO/ICB/commit/9c6b6357f8d8f10ceef74cd6832183bdc8cab590))
* Added CodeQL ([eeb3522](https://github.com/LogosITO/ICB/commit/eeb35225faa301c77787f994d4358800e90fb907))
* Added first implementation of new easy-to-use report system ([fc28dd3](https://github.com/LogosITO/ICB/commit/fc28dd330e29ed63717570c214ca312ddd7d43e5))
* Added first web part ([216f3f5](https://github.com/LogosITO/ICB/commit/216f3f54091e8ac5617b5bf9c3bdfa11a3ced9a6))
* Added icb-server implementation ([074f8a0](https://github.com/LogosITO/ICB/commit/074f8a00a19921d3accaa38454975908c26dd402))
* Autodocs ([f600b9d](https://github.com/LogosITO/ICB/commit/f600b9d910f137a0c8f2b40a5361df1a084ecbbc))
* **benches:** Expanded benchmarks with go and ruby supported languages ([c996221](https://github.com/LogosITO/ICB/commit/c9962214b91eab7da7dd5cfc50630b273e7162c3))
* **bench:** group benchmarks by crate, expand test sizes ([22942fb](https://github.com/LogosITO/ICB/commit/22942fb01428867677287b0f66d31c981595728e))
* First parser ([e0c7aab](https://github.com/LogosITO/ICB/commit/e0c7aab017d152af7dbe840697bf0089786fed67))
* full C++ support with compile_commands.json, parallel parsing, and CLI integration ([24992c1](https://github.com/LogosITO/ICB/commit/24992c1fd1a509dc26444bf0d12dee1798a9837e))
* high-performance C++ parser, graph benchmarks, UI fixes ([9f735e2](https://github.com/LogosITO/ICB/commit/9f735e2326783ee596cd073491f16a76489399eb))
* **icb-clang:** high‑performance parser with comprehensive benchmarks ([ebf87ed](https://github.com/LogosITO/ICB/commit/ebf87ed7ca0083dd642a2fefdda4d7074baf700f))
* improve benchmark JSON structure and dashboard ([33baeac](https://github.com/LogosITO/ICB/commit/33baeac5cde3f2c388ef8481ff2d4d5af95afe09))
* new scalable architecture with parser, graph core, and CLI ([1788a76](https://github.com/LogosITO/ICB/commit/1788a768ea31985b5a33cf2cbf99df4f25c4962f))
* parallel indexing, call graph queries, and DOT export ([d48a757](https://github.com/LogosITO/ICB/commit/d48a757b86e7e76f126225369225c6ac181627ee))
* Python support in server, fix Sigma import, drop icb-clang dependency ([c353613](https://github.com/LogosITO/ICB/commit/c353613b30823df552ac12a230aad704b7fdda0b))
* rebuild architecture with docs and tests ([3f12405](https://github.com/LogosITO/ICB/commit/3f124054b1d08740a04c0fb4959dbad93edb08b7))
* **server:** incremental fact caching with SHA-256 per-file hash ([8bac332](https://github.com/LogosITO/ICB/commit/8bac3322860d27d1cbad99db3521ffa7f030363e))
* universal heuristic parser for 98% of languages ([f883fda](https://github.com/LogosITO/ICB/commit/f883fda07cecee3efffb63aabe9e60f3c57ff060))
* Updated frontend ([d74b412](https://github.com/LogosITO/ICB/commit/d74b412d54214be7ce483755836786042af7ab86))
* Upgraded web implementation of graph ([3346b08](https://github.com/LogosITO/ICB/commit/3346b08cd8745c43fa6a3bc27a032a0489fd0116))
* upload ZIP, CORS fix, robust directory traversal, auto-detect language ([135d5a0](https://github.com/LogosITO/ICB/commit/135d5a014b139b6d6b948ee81e8ff209ebcc944d))
* working analytics dashboard + graph fixes + monochrome UI ([721d272](https://github.com/LogosITO/ICB/commit/721d272b40475e20cbfe91247b4454ec02611aae))
* working analytics server + dashboard foundation ([0f307f5](https://github.com/LogosITO/ICB/commit/0f307f5ad24913addf625662c3df96fe51901b56))


### Bug Fixes

* Added --exclude icb-clang in docs ([6bb586e](https://github.com/LogosITO/ICB/commit/6bb586e4b89c69cc8d8fb817b3f39101bc23dc28))
* cargo format for rust ([ac2b40e](https://github.com/LogosITO/ICB/commit/ac2b40eff52da1dadf7f1861dcad0282d82e712b))
* Changed parse_language function for supporting for all languages by heurestic parser ([19bf390](https://github.com/LogosITO/ICB/commit/19bf390d2145c8e0cb62868d94d2e7434de349fa))
* **ci:** Added clang install in ci ([aeb9fc1](https://github.com/LogosITO/ICB/commit/aeb9fc18bb08fca8ed849fac4208ae7033f7da85))
* **ci:** Changed github secret token to custom fine-grained token ([f0fd304](https://github.com/LogosITO/ICB/commit/f0fd30499deff3abf714419e4b7605e41ad499e9))
* **ci:** Changed release-please building style to monorepo ([1c2e9c9](https://github.com/LogosITO/ICB/commit/1c2e9c9421891a8c899b422a546773a2ce6d2d55))
* **ci:** Fixed Cargo.toml for new release-please github action ([3abc2cc](https://github.com/LogosITO/ICB/commit/3abc2cc5e488ff3ff5f7203b0db6233e7924cebb))
* **ci:** Fixed problem in ci with CLand/LLVM installing ([51721e0](https://github.com/LogosITO/ICB/commit/51721e037f8be016309a2eb141edfdac0c38d70f))
* **ci:** Hotfix ([21b12bd](https://github.com/LogosITO/ICB/commit/21b12bd6f9b061c2d464ddc468db4845d0b231e9))
* **ci:** Hotfix ([c8927e3](https://github.com/LogosITO/ICB/commit/c8927e365020b48e4dd22f563452c04d2542b103))
* **ci:** Updated release-please yml ([f1117ec](https://github.com/LogosITO/ICB/commit/f1117ecf470a5bce30fd988e01f3d32a9639c52e))
* correct Clang integration, noise filtering, and server stability ([1f8c675](https://github.com/LogosITO/ICB/commit/1f8c675d351e43a5ff63e49d9cf6ddf88e3016f9))
* correct deny.toml for cargo-deny ([8d52221](https://github.com/LogosITO/ICB/commit/8d52221a484cc9b222448d147f2d470d1bbb0804))
* Deleted GraphViewer (switched GraphViewer to TreeViewer) ([9d36e1a](https://github.com/LogosITO/ICB/commit/9d36e1a9f0bf9fd98ee246221f4e653ca1b305c8))
* **docs:** CodeQL vulnerability fix ([07132e5](https://github.com/LogosITO/ICB/commit/07132e5cc09e51fe8c44e9b0c4af60393f28aeb9))
* drag-n-drop field fixed in Overview tab ([920ae3c](https://github.com/LogosITO/ICB/commit/920ae3c43aba6b80de7c2dac19e29dc206e5400c))
* empty callees/callers, optimize metrics, add elk call tree ([835f150](https://github.com/LogosITO/ICB/commit/835f1500da970e80a7cdbbaef135cd6490c2c6b9))
* expanded caching for clang pipeline ([641419d](https://github.com/LogosITO/ICB/commit/641419d369378d10964510213622498029eedac5))
* Fix --no-system-headers flag ([440f513](https://github.com/LogosITO/ICB/commit/440f513bf1e0bb63f91face595e03a73fff569dd))
* Fixed benchmark starting scenaries ([ddf6fd5](https://github.com/LogosITO/ICB/commit/ddf6fd59e02d963a958243730a1b00f47dad80a0))
* Fixed builder.rs file ([fee5338](https://github.com/LogosITO/ICB/commit/fee53388b6db3ae2924b13f66fad3ec2d31d6507))
* Fixed icb-lang/src/lib.rs ([3624632](https://github.com/LogosITO/ICB/commit/362463268042f5e4b6052a38f5298c5be1bd4017))
* Go/Ruby/Python parser tests after refactor ([8582245](https://github.com/LogosITO/ICB/commit/85822452d86e1a399f1f64a23756d2e9afe6d68a))
* Heurestic parser algorythm changed from tokenizator to token counter + regex ([bc48094](https://github.com/LogosITO/ICB/commit/bc48094ac36df7a7afc831186ec03b7a37aa56b6))
* Heuristic_parser.rs fixed for go test project ([2cd78ea](https://github.com/LogosITO/ICB/commit/2cd78ea5bf9b434be26698d4109de77c9f9b1e1c))
* Hotfix ([801c2e1](https://github.com/LogosITO/ICB/commit/801c2e1dbe10186033279998087cef53e8dfb6b8))
* Hotfix ([5a227ec](https://github.com/LogosITO/ICB/commit/5a227ec5e96e59db9d64fbc7101da41d3e6ebeb4))
* Hotfix ([2c8654e](https://github.com/LogosITO/ICB/commit/2c8654ea5ff850a592acd968427a9f5743ac1246))
* Hotfix ([38c2422](https://github.com/LogosITO/ICB/commit/38c2422b680be7daaea18c877cd2396ec334f954))
* Hotfix ([b181224](https://github.com/LogosITO/ICB/commit/b1812246fdcda30a3405b80056a53ff9ea3e93be))
* Hotfix ([8ff6393](https://github.com/LogosITO/ICB/commit/8ff6393783f78d46c8a2ffae075f140bd4484e5c))
* Hotfix ([ac428fd](https://github.com/LogosITO/ICB/commit/ac428fd17518080abbf765dea79744034f9cb1fe))
* Hotfix ([e575cd6](https://github.com/LogosITO/ICB/commit/e575cd6cf1895175c00a87bbb3ff134aeac4312c))
* Hotfix 2 ([b90d9a4](https://github.com/LogosITO/ICB/commit/b90d9a469f6123979a9be9fb41f03d775e1df0ec))
* Hotfix 2 ([139d6f7](https://github.com/LogosITO/ICB/commit/139d6f75ba9834bc9a0c666619790550fd9ade45))
* Hotfix 3 ([2baeea8](https://github.com/LogosITO/ICB/commit/2baeea858141b5acb8ab662620398dff6ecea507))
* Hotfix 3 ([5ee6264](https://github.com/LogosITO/ICB/commit/5ee6264c542665269e61f241a2b2a29732734a4d))
* Hotfix 3 ([07586e2](https://github.com/LogosITO/ICB/commit/07586e2db4d5ecab2f23bc7a43d46f4661070095))
* Hotfix 4 ([e664946](https://github.com/LogosITO/ICB/commit/e664946feada9b1449bcfbf242192b92b80debac))
* Hotfix 4 ([d8e9c87](https://github.com/LogosITO/ICB/commit/d8e9c87fa3630429129878fabea06decf15aea55))
* Hotfix2 ([07507f5](https://github.com/LogosITO/ICB/commit/07507f5f8ff9aebed5b0dbbc070c940383e8b7ab))
* **icb-server:** convert Clang USR to readable names in graph ([8794334](https://github.com/LogosITO/ICB/commit/879433462dbbba55cd63dcd1c8f7055f0c59ba5d))
* improve tree viewer rendering, edge routing, and network resilience ([725d6a4](https://github.com/LogosITO/ICB/commit/725d6a4d769c0b2a21ae6450792e58013230f3db))
* index.html for docs ([ae1b78f](https://github.com/LogosITO/ICB/commit/ae1b78f17310a59b3b2f788abe9e02f827a5ad2f))
* Install clang fix ([71b0bce](https://github.com/LogosITO/ICB/commit/71b0bce4914959a78515694139878cfb415abdf5))
* multi-language selection, strict extension filtering, class extraction fallback ([5886b01](https://github.com/LogosITO/ICB/commit/5886b01682efa1cf32431a49599109415ee80eab))
* **parser:** add Rust parser using tree-sitter-rust ([2c80da2](https://github.com/LogosITO/ICB/commit/2c80da206a7ebcb48bff1d27684eb91b17d1a29b))
* prevent infinite loop in detect_complex_functions, add recursion limit ([a3f0ab8](https://github.com/LogosITO/ICB/commit/a3f0ab832371af46fe5955e0359b4bba78f4edbf))
* prevent infinite loop in detect_complex_functions, add recursion limit ([7108e8e](https://github.com/LogosITO/ICB/commit/7108e8ec32e723704df7be8b53f9adf9744fb5a0))
* release-please ci ([45b6623](https://github.com/LogosITO/ICB/commit/45b662390142a968f39b35c580d782ef6410ef02))
* remove collapse/expand buttons, keep debounced hover and search in TreeViewer ([ef026c6](https://github.com/LogosITO/ICB/commit/ef026c6784b21caae44360f3b2cb75c7df2c1c94))
* Removed audit.yml bc i cant setup it ([d9330a5](https://github.com/LogosITO/ICB/commit/d9330a5a500cff3e098fd5747abee64546e8bdfc))
* reset release-please manifest state ([9e6e834](https://github.com/LogosITO/ICB/commit/9e6e8340f11f93fc608c5dc246bb5a28d9cfa261))
* resolve unknown backends in benchmarks and improve layout ([8eb215d](https://github.com/LogosITO/ICB/commit/8eb215d931be3972829074c7668bcf782ff72a2e))
* resolve unknown backends in benchmarks and improve layout ([5e7beff](https://github.com/LogosITO/ICB/commit/5e7beffbd87195ceb8d9625cda703da47a6d2de3))
* return complexity analysis, add LOC column, fix Clang body location ([1d311e6](https://github.com/LogosITO/ICB/commit/1d311e6577535c40ee87338182fcb05d126e195e))
* robust ZIP upload with Clang, clean output for C++ projects ([e758b3a](https://github.com/LogosITO/ICB/commit/e758b3ac8054ae77a0cfc29e21782c84ada98e8b))
* **server:** replace linear name search with HashMap in analytics ([961de61](https://github.com/LogosITO/ICB/commit/961de61820a0727f7769664133fa4e402cac0596))
* simple_function test in icb-clang parser tests ([078a80a](https://github.com/LogosITO/ICB/commit/078a80acf61bc50b54fa9f4c69f2a9d623cf3b98))
* Tests fixed in lang/python.rs ([8fe9ca7](https://github.com/LogosITO/ICB/commit/8fe9ca721c93fcba116689fd71c66422c6000993))
* tree view with ELK layout, stable IDs and chain coloring ([6e01b7f](https://github.com/LogosITO/ICB/commit/6e01b7f3ee8ad36eae9f6d68d5cbfa379772588b))
* update dead code test to reflect unreachable self‑loop function ([8f897d5](https://github.com/LogosITO/ICB/commit/8f897d5c4317a5a96d81a01595b6fd2a391170d7))
* **web:** virtual scroll in tables, fit-to-screen and neighbor highlight in TreeViewer ([7882328](https://github.com/LogosITO/ICB/commit/7882328f0105c00d8781a0b2de00246c1edc1099))


### Performance Improvements

* **icb-clang:** optimize parser, add benchmarks, fix graph names ([33b9b52](https://github.com/LogosITO/ICB/commit/33b9b529e647ca27bbd0979e658361879978a7a3))


### Documentation

* **ci:** fixed docs.yml ([60f957e](https://github.com/LogosITO/ICB/commit/60f957e62cc0f8a0c9c8a43e280dcec4ca55c480))
* **cli:** Updated autodocs content in cli ([e3dd339](https://github.com/LogosITO/ICB/commit/e3dd3395025ad03137916eeee4e0f06d7a6add35))
* update README with English version highlighting completed features ([edd26ac](https://github.com/LogosITO/ICB/commit/edd26acbea87746897d997de7e0d8ba6969b7047))
* **web:** add user guide page and link from index ([3305a8a](https://github.com/LogosITO/ICB/commit/3305a8a4352b600df7d490aaf50ca84eeb5e50af))


### Chores

* add chore type to changelog sections ([2eb543e](https://github.com/LogosITO/ICB/commit/2eb543eac4a884a4d985bc7781ced9afe854e3f6))
* add CLI diff command ([e61d840](https://github.com/LogosITO/ICB/commit/e61d840d002865cce6af9de3fdc1386fc9e4a1c6))
* add icb-rustc benchmarks and fix graph_builder clippy warnings ([9b489dd](https://github.com/LogosITO/ICB/commit/9b489ddb787fd52760aa825371513ecb4d011584))
* add icb-rustc crate with nightly feature for rustc-based analysis ([7832f03](https://github.com/LogosITO/ICB/commit/7832f0315748a6099f36f45b09ce73142af2033a))
* **benches:** Another session of fixing 11 ns benchmark output ([4230aec](https://github.com/LogosITO/ICB/commit/4230aec254852dceb71fd2ca4b3074b93164bcaa))
* **benches:** Another try to fight with built-in optimizations and cache ([cd2b753](https://github.com/LogosITO/ICB/commit/cd2b753fc85ee6aa99c33f0f4672748028754222))
* **benches:** Fixed parsing script and icb-rustc benches ([ca154d7](https://github.com/LogosITO/ICB/commit/ca154d7da44f924d3bbb866d38aaf156cb4a0758))
* **ci:** Expanded pre-commit using scenarios ([97e4808](https://github.com/LogosITO/ICB/commit/97e48089120c6d0d38bf0a5e9e535863e131cfcc))
* **ci:** release-please fix ([1622369](https://github.com/LogosITO/ICB/commit/1622369aaa9696f06b04db3e46e5db3142282a7c))
* Deleted unused files and added .gittatributes file ([a85ac64](https://github.com/LogosITO/ICB/commit/a85ac64791545a718d4674577ee8527d4cc58cf1))
* fix bench classification + make analytics public + update web pages ([194b135](https://github.com/LogosITO/ICB/commit/194b135010f445deeedd5ed153271d64e4aa273d))
* fix benchmark json generation and CI parsing ([8764631](https://github.com/LogosITO/ICB/commit/876463165d1e9c6d7ac6a69d8f07cfe39bdf49ab))
* Hotfix ([1218fdf](https://github.com/LogosITO/ICB/commit/1218fdfc4ce253128f625bc58e6bc2bed9175b53))
* Hotfix ([fe2d0ae](https://github.com/LogosITO/ICB/commit/fe2d0ae3a94da6ce49dd669aaad48706de03779b))
* Hotfix 2 ([27e90ff](https://github.com/LogosITO/ICB/commit/27e90ff0b0eedd6ddc31b2533a947ac74a3e5a6b))
* **main:** release 0.1.0 ([c02a0e4](https://github.com/LogosITO/ICB/commit/c02a0e427d5d67d517182040252780ae0aefad13))
* **main:** release 0.1.0 ([b79b6cc](https://github.com/LogosITO/ICB/commit/b79b6cc739150d4a5b570c4fd5cc1de460fda4c7))
* **main:** release 0.2.0 ([f9de95b](https://github.com/LogosITO/ICB/commit/f9de95b7e26557683f6775f64f9cb72648f23a40))
* **main:** release 0.2.0 ([9a47af7](https://github.com/LogosITO/ICB/commit/9a47af7787b60eed643409b0d0f8f4c4a1659a8e))
* **main:** release 0.3.0 ([5a14568](https://github.com/LogosITO/ICB/commit/5a1456898aa18ccdba0650a1ce6f1f8c48cd7199))
* **main:** release 0.3.0 ([74d9efd](https://github.com/LogosITO/ICB/commit/74d9efd04b2c3d735a3a3458dfbd6196d37b12b7))
* **main:** release 0.4.0 ([33b7e5c](https://github.com/LogosITO/ICB/commit/33b7e5cb9ee46a3df329532fb5a120de4830ead1))
* **main:** release 0.4.0 ([693d479](https://github.com/LogosITO/ICB/commit/693d479a29108b5cf2e021854cd42f1df8f9fb71))
* **main:** release 0.4.1 ([73a7eda](https://github.com/LogosITO/ICB/commit/73a7eda8c5300f76c702e738991eb3afc15379ff))
* **main:** release 0.4.1 ([a14eafd](https://github.com/LogosITO/ICB/commit/a14eafdfc0d2d0f5fd2562ac799187c9d04f6661))
* **main:** release 0.4.2 ([8723fef](https://github.com/LogosITO/ICB/commit/8723fef80fc492747cb85f3c7cf1b286dbeaa648))
* **main:** release 0.4.2 ([c535bb4](https://github.com/LogosITO/ICB/commit/c535bb40006ff35704b45503bbbe56cdd6c621b5))
* **main:** release 0.4.3 ([cf9e3e2](https://github.com/LogosITO/ICB/commit/cf9e3e2db2566b0c6a783bc51752d22016e4965b))
* **main:** release 0.4.3 ([5e978ca](https://github.com/LogosITO/ICB/commit/5e978cafd9a4121b6391c820796da733fa05a966))
* **main:** release 0.4.4 ([23ae7d9](https://github.com/LogosITO/ICB/commit/23ae7d90a9d1c30ebe763cb0cb079f13c6ff106a))
* **main:** release 0.4.4 ([9c751fb](https://github.com/LogosITO/ICB/commit/9c751fbc9a17ca4afa8b2248ef070c090aad2a25))
* **main:** release 0.5.0 ([7f7c5ba](https://github.com/LogosITO/ICB/commit/7f7c5bacaa1870bbd458348c1eb3c07928e4ed7c))
* **main:** release 0.5.0 ([0cd1f17](https://github.com/LogosITO/ICB/commit/0cd1f1729849b9a96ef660ad80e18ff9a6e15245))
* **main:** release 0.5.1 ([9b099ac](https://github.com/LogosITO/ICB/commit/9b099ac023342bdec7b9f8b0c2e180e4d0099304))
* **main:** release 0.5.1 ([cb5988c](https://github.com/LogosITO/ICB/commit/cb5988c3da7ec22d1738a044ead68b5be9c211e9))
* **main:** release 0.5.2 ([37c453b](https://github.com/LogosITO/ICB/commit/37c453b33d5a6ee4e97a195ae0ec62e30c711399))
* **main:** release 0.5.2 ([d44b9e1](https://github.com/LogosITO/ICB/commit/d44b9e10c174e52f1e6feb1dd093170a78a18c51))
* **main:** release 0.5.3 ([0078087](https://github.com/LogosITO/ICB/commit/00780874bc468d552cd4ccfb218c082ec1e24fb3))
* **main:** release 0.5.3 ([888a233](https://github.com/LogosITO/ICB/commit/888a233e070269e43501e244a3a28ce8091d927a))
* **main:** release 0.6.0 ([9f6528e](https://github.com/LogosITO/ICB/commit/9f6528e3154b1c2a47aa3adae311af1540eb4c90))
* **main:** release 0.6.0 ([fe495fb](https://github.com/LogosITO/ICB/commit/fe495fbbe2266068d459eba0c12b68db0d2f5732))
* **main:** release 0.6.1 ([ec2bbdc](https://github.com/LogosITO/ICB/commit/ec2bbdc4f94ec31def25e69188cb1cdb1bb2a132))
* **main:** release 0.6.1 ([52eecad](https://github.com/LogosITO/ICB/commit/52eecadff06bb496e1e2e6f36b9fa7fcccdcd745))
* **main:** release 0.6.2 ([bc48694](https://github.com/LogosITO/ICB/commit/bc4869422e127da3a099558267f843e96be5ac02))
* **main:** release 0.6.2 ([8c77423](https://github.com/LogosITO/ICB/commit/8c77423bf71e4bd773bcc52e2b99b54efbf0029b))
* **main:** release 0.7.0 ([786bb25](https://github.com/LogosITO/ICB/commit/786bb25fed1adf86b952f7d09179576ee66e701b))
* **main:** release 0.7.0 ([d56d5cf](https://github.com/LogosITO/ICB/commit/d56d5cfd0df595faa9d57c310681c66fcfb3bbf5))
* **main:** release 0.7.1 ([a30ba67](https://github.com/LogosITO/ICB/commit/a30ba67f5baaab23be53e46736ee40d8e0b6a0f4))
* **main:** release 0.7.1 ([791e3ad](https://github.com/LogosITO/ICB/commit/791e3ad8febfec4240d450c360d8f06ac7bc639c))
* **main:** release 0.7.2 ([510b651](https://github.com/LogosITO/ICB/commit/510b651b12ce2dc9fe4eab68d6adc84f759e2d01))
* **main:** release 0.7.2 ([1a335a4](https://github.com/LogosITO/ICB/commit/1a335a4ba680c49cb9b8debffd169709b5c55fb1))
* **main:** release 0.8.0 ([e219ebb](https://github.com/LogosITO/ICB/commit/e219ebbceaedb057ddbfd80756b7a8e8e38705d9))
* **main:** release 0.8.0 ([629a508](https://github.com/LogosITO/ICB/commit/629a5088b2778e40ae7ed14e906407fe49b8efed))
* **main:** release 0.8.1 ([22b3283](https://github.com/LogosITO/ICB/commit/22b3283148ca2295fe417f74c3808a5505e12a70))
* **main:** release 0.8.1 ([04d75c0](https://github.com/LogosITO/ICB/commit/04d75c0e4339deeb0f4048b8ced555ca463eba65))
* **main:** release 0.8.10 ([d5933d6](https://github.com/LogosITO/ICB/commit/d5933d62a0211db575a3bacb1364f841d5957d7c))
* **main:** release 0.8.10 ([006fcad](https://github.com/LogosITO/ICB/commit/006fcad26087822f54979f0acf01fdbc9ca77435))
* **main:** release 0.8.11 ([7986881](https://github.com/LogosITO/ICB/commit/7986881e124e599a9e845c1db7343c2fecf9b9a8))
* **main:** release 0.8.11 ([50d369e](https://github.com/LogosITO/ICB/commit/50d369ecabe63a9933de1c76671ed78531587309))
* **main:** release 0.8.12 ([df0252c](https://github.com/LogosITO/ICB/commit/df0252c7f5a702afec0e7f5bcee2fe68c9da9435))
* **main:** release 0.8.12 ([81bf85e](https://github.com/LogosITO/ICB/commit/81bf85e475a07c05f4a72210ff605e6d0e8c1046))
* **main:** release 0.8.2 ([cc473dc](https://github.com/LogosITO/ICB/commit/cc473dcdb0eac54a5e1ccc9907d11ffbd272bf0c))
* **main:** release 0.8.2 ([db8ccd1](https://github.com/LogosITO/ICB/commit/db8ccd13e00a636db1bc8ab6a952b0a1471b3f11))
* **main:** release 0.8.3 ([e00bfcb](https://github.com/LogosITO/ICB/commit/e00bfcb7c03974752df15f9289ea645bb5c9e636))
* **main:** release 0.8.3 ([6aef9aa](https://github.com/LogosITO/ICB/commit/6aef9aa4bfb5fff0bc9415c98c71ff9f1903237b))
* **main:** release 0.8.4 ([faca0a8](https://github.com/LogosITO/ICB/commit/faca0a8df4a630a50e0eed68c389735cd9c14a4f))
* **main:** release 0.8.4 ([03db4cb](https://github.com/LogosITO/ICB/commit/03db4cb7e7e324f1cdb8cb81887f1607c076f368))
* **main:** release 0.8.5 ([2689cae](https://github.com/LogosITO/ICB/commit/2689cae5b8f7b26033c909f430102bca2e720c33))
* **main:** release 0.8.5 ([9999b4b](https://github.com/LogosITO/ICB/commit/9999b4b6c49c3a222b7db2b99e65741950366c95))
* **main:** release 0.8.6 ([5652ec0](https://github.com/LogosITO/ICB/commit/5652ec02e27e92f64c44bd53dc9e0a4ffd9dc720))
* **main:** release 0.8.7 ([c6db88f](https://github.com/LogosITO/ICB/commit/c6db88f73bfb08dcba4167aafb5e8d39715e37cc))
* **main:** release 0.8.7 ([771e01f](https://github.com/LogosITO/ICB/commit/771e01f6e9606475f2d7f5ffdcc12e1569eb66d3))
* **main:** release 0.8.8 ([1e7f0bb](https://github.com/LogosITO/ICB/commit/1e7f0bb1e38e3033250c303988c0e973697085f1))
* **main:** release 0.8.8 ([7688138](https://github.com/LogosITO/ICB/commit/7688138ac3639cf9997f5bae3fc7d6b44f954882))
* **main:** release 0.8.9 ([d9f07c4](https://github.com/LogosITO/ICB/commit/d9f07c46018d58cc6f64c3d52f210a1aadf5f43b))
* **main:** release 0.8.9 ([5dd1b5f](https://github.com/LogosITO/ICB/commit/5dd1b5f1d0b5b0cae8f8d04b41c42c02a137b21c))
* **parser:** unify language parsers with common traversal utilities ([4194eeb](https://github.com/LogosITO/ICB/commit/4194eebfea8aadea5a101642455618b563183ae3))
* properly parse criterion benchmark output ([01facb1](https://github.com/LogosITO/ICB/commit/01facb122e26900fbdf6c96ba43ce55e5a94fe1a))
* release main ([c7ec737](https://github.com/LogosITO/ICB/commit/c7ec737e33881c52dc1743f3bdb72603f90b69f6))
* release main ([1f92f20](https://github.com/LogosITO/ICB/commit/1f92f20df9fb396cda298a9f56d5d0d2efec845d))
* **rustc:** upgrade icb-rustc to production-ready HIR visitor ([ec5aaea](https://github.com/LogosITO/ICB/commit/ec5aaea108698d09f7a200de03c39b07b1dddd6f))
* SECURITY.md fix ([b7de2f7](https://github.com/LogosITO/ICB/commit/b7de2f73becdf6629059dc8cc8ac16db0619cdbe))
* **web:** unify diff viewer, virtualize tables, enhance tree viewer ([9c9cf52](https://github.com/LogosITO/ICB/commit/9c9cf5200b5f0e7cf9e8baa402bbff822b1cebee))

## [0.8.12](https://github.com/LogosITO/ICB/compare/v0.8.11...v0.8.12) (2026-05-10)


### Bug Fixes

* Hotfix ([801c2e1](https://github.com/LogosITO/ICB/commit/801c2e1dbe10186033279998087cef53e8dfb6b8))

## [0.8.11](https://github.com/LogosITO/ICB/compare/v0.8.10...v0.8.11) (2026-05-09)


### Bug Fixes

* Hotfix 3 ([2baeea8](https://github.com/LogosITO/ICB/commit/2baeea858141b5acb8ab662620398dff6ecea507))
* Hotfix 4 ([e664946](https://github.com/LogosITO/ICB/commit/e664946feada9b1449bcfbf242192b92b80debac))

## [0.8.10](https://github.com/LogosITO/ICB/compare/v0.8.9...v0.8.10) (2026-05-08)


### Bug Fixes

* **web:** virtual scroll in tables, fit-to-screen and neighbor highlight in TreeViewer ([7882328](https://github.com/LogosITO/ICB/commit/7882328f0105c00d8781a0b2de00246c1edc1099))

## [0.8.9](https://github.com/LogosITO/ICB/compare/v0.8.8...v0.8.9) (2026-05-07)


### Bug Fixes

* cargo format for rust ([ac2b40e](https://github.com/LogosITO/ICB/commit/ac2b40eff52da1dadf7f1861dcad0282d82e712b))

## [0.8.8](https://github.com/LogosITO/ICB/compare/v0.8.7...v0.8.8) (2026-05-07)


### Bug Fixes

* index.html for docs ([ae1b78f](https://github.com/LogosITO/ICB/commit/ae1b78f17310a59b3b2f788abe9e02f827a5ad2f))

## [0.8.7](https://github.com/LogosITO/ICB/compare/v0.8.6...v0.8.7) (2026-05-07)


### Bug Fixes

* **parser:** add Rust parser using tree-sitter-rust ([2c80da2](https://github.com/LogosITO/ICB/commit/2c80da206a7ebcb48bff1d27684eb91b17d1a29b))

## [0.8.6](https://github.com/LogosITO/ICB/compare/v0.8.5...v0.8.6) (2026-05-06)


### Bug Fixes

* improve tree viewer rendering, edge routing, and network resilience ([725d6a4](https://github.com/LogosITO/ICB/commit/725d6a4d769c0b2a21ae6450792e58013230f3db))

## [0.8.5](https://github.com/LogosITO/ICB/compare/v0.8.4...v0.8.5) (2026-05-06)


### Bug Fixes

* drag-n-drop field fixed in Overview tab ([920ae3c](https://github.com/LogosITO/ICB/commit/920ae3c43aba6b80de7c2dac19e29dc206e5400c))

## [0.8.4](https://github.com/LogosITO/ICB/compare/v0.8.3...v0.8.4) (2026-05-06)


### Bug Fixes

* Deleted GraphViewer (switched GraphViewer to TreeViewer) ([9d36e1a](https://github.com/LogosITO/ICB/commit/9d36e1a9f0bf9fd98ee246221f4e653ca1b305c8))

## [0.8.3](https://github.com/LogosITO/ICB/compare/v0.8.2...v0.8.3) (2026-05-06)


### Bug Fixes

* empty callees/callers, optimize metrics, add elk call tree ([835f150](https://github.com/LogosITO/ICB/commit/835f1500da970e80a7cdbbaef135cd6490c2c6b9))

## [0.8.2](https://github.com/LogosITO/ICB/compare/v0.8.1...v0.8.2) (2026-05-06)


### Bug Fixes

* remove collapse/expand buttons, keep debounced hover and search in TreeViewer ([ef026c6](https://github.com/LogosITO/ICB/commit/ef026c6784b21caae44360f3b2cb75c7df2c1c94))

## [0.8.1](https://github.com/LogosITO/ICB/compare/v0.8.0...v0.8.1) (2026-05-06)


### Bug Fixes

* tree view with ELK layout, stable IDs and chain coloring ([6e01b7f](https://github.com/LogosITO/ICB/commit/6e01b7f3ee8ad36eae9f6d68d5cbfa379772588b))

## [0.8.0](https://github.com/LogosITO/ICB/compare/v0.7.2...v0.8.0) (2026-05-06)


### Features

* add tree visualization, removing graph explosion fallback and adding safety limits ([a275ddc](https://github.com/LogosITO/ICB/commit/a275ddcd13164434ecdff9213178c3025d974a80))

## [0.7.2](https://github.com/LogosITO/ICB/compare/v0.7.1...v0.7.2) (2026-05-05)


### Bug Fixes

* Go/Ruby/Python parser tests after refactor ([8582245](https://github.com/LogosITO/ICB/commit/85822452d86e1a399f1f64a23756d2e9afe6d68a))

## [0.7.1](https://github.com/LogosITO/ICB/compare/v0.7.0...v0.7.1) (2026-05-05)


### Bug Fixes

* **server:** replace linear name search with HashMap in analytics ([961de61](https://github.com/LogosITO/ICB/commit/961de61820a0727f7769664133fa4e402cac0596))

## [0.7.0](https://github.com/LogosITO/ICB/compare/v0.6.2...v0.7.0) (2026-05-05)


### Features

* **bench:** group benchmarks by crate, expand test sizes ([22942fb](https://github.com/LogosITO/ICB/commit/22942fb01428867677287b0f66d31c981595728e))

## [0.6.2](https://github.com/LogosITO/ICB/compare/v0.6.1...v0.6.2) (2026-05-05)


### Bug Fixes

* expanded caching for clang pipeline ([641419d](https://github.com/LogosITO/ICB/commit/641419d369378d10964510213622498029eedac5))

## [0.6.1](https://github.com/LogosITO/ICB/compare/v0.6.0...v0.6.1) (2026-05-05)


### Bug Fixes

* simple_function test in icb-clang parser tests ([078a80a](https://github.com/LogosITO/ICB/commit/078a80acf61bc50b54fa9f4c69f2a9d623cf3b98))

## [0.6.0](https://github.com/LogosITO/ICB/compare/v0.5.3...v0.6.0) (2026-05-05)


### Features

* **server:** incremental fact caching with SHA-256 per-file hash ([8bac332](https://github.com/LogosITO/ICB/commit/8bac3322860d27d1cbad99db3521ffa7f030363e))

## [0.5.3](https://github.com/LogosITO/ICB/compare/v0.5.2...v0.5.3) (2026-05-05)


### Bug Fixes

* return complexity analysis, add LOC column, fix Clang body location ([1d311e6](https://github.com/LogosITO/ICB/commit/1d311e6577535c40ee87338182fcb05d126e195e))

## [0.5.2](https://github.com/LogosITO/ICB/compare/v0.5.1...v0.5.2) (2026-05-05)


### Bug Fixes

* resolve unknown backends in benchmarks and improve layout ([8eb215d](https://github.com/LogosITO/ICB/commit/8eb215d931be3972829074c7668bcf782ff72a2e))

## [0.5.1](https://github.com/LogosITO/ICB/compare/v0.5.0...v0.5.1) (2026-05-05)


### Bug Fixes

* resolve unknown backends in benchmarks and improve layout ([5e7beff](https://github.com/LogosITO/ICB/commit/5e7beffbd87195ceb8d9625cda703da47a6d2de3))

## [0.5.0](https://github.com/LogosITO/ICB/compare/v0.4.4...v0.5.0) (2026-05-05)


### Features

* improve benchmark JSON structure and dashboard ([33baeac](https://github.com/LogosITO/ICB/commit/33baeac5cde3f2c388ef8481ff2d4d5af95afe09))

## [0.4.4](https://github.com/LogosITO/ICB/compare/v0.4.3...v0.4.4) (2026-05-05)


### Bug Fixes

* robust ZIP upload with Clang, clean output for C++ projects ([e758b3a](https://github.com/LogosITO/ICB/commit/e758b3ac8054ae77a0cfc29e21782c84ada98e8b))

## [0.4.3](https://github.com/LogosITO/ICB/compare/v0.4.2...v0.4.3) (2026-05-05)


### Bug Fixes

* correct Clang integration, noise filtering, and server stability ([1f8c675](https://github.com/LogosITO/ICB/commit/1f8c675d351e43a5ff63e49d9cf6ddf88e3016f9))

## [0.4.2](https://github.com/LogosITO/ICB/compare/v0.4.1...v0.4.2) (2026-05-05)


### Bug Fixes

* release-please ci ([45b6623](https://github.com/LogosITO/ICB/commit/45b662390142a968f39b35c580d782ef6410ef02))

## [0.4.1](https://github.com/LogosITO/ICB/compare/v0.4.0...v0.4.1) (2026-05-05)


### Bug Fixes

* multi-language selection, strict extension filtering, class extraction fallback ([5886b01](https://github.com/LogosITO/ICB/commit/5886b01682efa1cf32431a49599109415ee80eab))

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
