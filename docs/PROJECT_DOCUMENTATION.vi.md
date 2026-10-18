# 1. Thông tin tài liệu và tóm tắt dự án

## 1.1. Mục đích và phạm vi

ReconX AI là ứng dụng desktop và CLI phục vụ quản lý hoạt động nghiên cứu bảo mật, phân tích dữ liệu cục bộ, lưu bằng chứng và soạn báo cáo. Hai giao diện sử dụng cùng Rust Core, SQLite và các quy tắc kiểm soát thực thi. Định hướng sản phẩm là kết nối Target → Scope → Job → Analysis → Evidence → Finding → Report, đồng thời cho phép dùng AI có cấu hình và phê duyệt rõ ràng để giải thích các kết quả thực tế.

Tài liệu này tổng hợp sản phẩm hiện hữu tại D:\ReconXAI, không mô tả một bản thiết kế giả định như thể đã được triển khai. Phạm vi gồm yêu cầu, kiến trúc, cấu trúc source, dữ liệu, hướng dẫn Desktop/CLI, AI, MCP, công cụ, bảo mật, kiểm thử, build, vận hành, artifact và các điều kiện còn thiếu để phát hành. Các bảng danh mục và phụ lục CLI được trích trực tiếp từ source/executable thay vì nhập thủ công tên chức năng tưởng tượng.

Phiên bản ứng dụng hiện tại là **0.1.0**. Mục tiêu **v1.0 chưa hoàn thành**. Kết quả local regression tốt không thay thế kiểm chứng Linux Desktop, clean installer, tài khoản cloud thật, các module còn thiếu hoặc giải quyết dependency đang bị chặn. Đây là tài liệu kỹ thuật và hướng dẫn sử dụng của development build, không phải chứng nhận sản phẩm đã production-ready.

## 1.2. Đối tượng sử dụng

- Người nghiên cứu: quản lý project/target, đọc HAR/OpenAPI, phân tích file, ghi nhận nhận xét và xuất báo cáo đã kiểm duyệt.
- Lập trình viên: mở rộng typed service trong Rust Core, tích hợp frontend/CLI mà không lặp business logic.
- QA: chạy fixture cục bộ, migration/persistence tests, native UI acceptance và đối chiếu hash của đúng binary.
- Người vận hành hoặc reviewer: đánh giá nơi lưu dữ liệu, kho credential, giới hạn quyền, dependency và các release gate.

Người vận hành phải sở hữu hoặc được phép sử dụng dữ liệu, file và dịch vụ thử nghiệm. Việc đăng ký một target, lưu credential hoặc chọn một mode không đồng nghĩa mọi loại phân tích đã được cài đặt hay được cấp quyền thực thi.

## 1.3. Quy ước trạng thái

| Trạng thái | Ý nghĩa trong tài liệu |
| --- | --- |
| NOT_STARTED | Chưa có workflow backend chuyên biệt đủ để sử dụng |
| PARTIAL / IN_PROGRESS | Có phần làm việc thật nhưng còn yêu cầu chức năng hoặc acceptance chưa đạt |
| IMPLEMENTED | Có code; chưa đủ bằng chứng chạy thực tế cho toàn bộ phạm vi |
| TESTED / RUNTIME_VERIFIED | Có test thực thi cho đúng phạm vi và nền tảng được chỉ rõ |
| MOCK_VERIFIED | Đã test protocol hoặc workflow với mock model xác định; không phải live cloud |
| UNVERIFIED | Chưa có bằng chứng runtime phù hợp |
| BLOCKED | Có điều kiện bên ngoài hoặc vấn đề kỹ thuật ngăn acceptance/release |

Mọi trạng thái phải đọc kèm phạm vi. Ví dụ, “Linux CLI TESTED trên Parrot/WSL2” không chứng minh Linux GUI, Secret Service hoặc mọi distribution. Một module PARTIAL có thể chứa một vertical slice đã RUNTIME_VERIFIED; điều đó không nâng toàn bộ module thành hoàn chỉnh.

# 2. Yêu cầu sản phẩm và hiện trạng chức năng

## 2.1. Luồng nghiệp vụ đã kết nối

Người dùng tạo project, đăng ký target hoặc import file. Core kiểm tra dữ liệu và scope. Các tác vụ được cho phép đi qua Job Engine, tạo kết quả có nguồn gốc và artifact hash. Evidence giữ liên kết đến target/job; finding bổ sung đánh giá của người nghiên cứu; report chụp một snapshot đã redaction và chỉ xuất khi người dùng duyệt đúng hash.

Đối với Web/API, dữ liệu bắt đầu từ HAR/OpenAPI offline. Người dùng có thể xem traffic, so sánh captures, đánh giá schema và dùng một request editor tách biệt để gửi từng request được duyệt đến loopback. Import, chỉnh sửa, lưu draft hoặc lưu identity không tự tạo lưu lượng mạng.

## 2.2. Ma trận phạm vi chính

| Nhóm | Hiện có | Giới hạn quan trọng |
| --- | --- | --- |
| Project / Target / Scope | CRUD, import/export, validate, persistence chung | Đăng ký 17 loại không đồng nghĩa có 17 analyzer |
| Reverse | Import, hash, PE/ELF/thin Mach-O, strings, cấu trúc, history | Không có decompiler/disassembler/debugger hoàn chỉnh |
| Jobs | Queue, hai worker tối đa, lease, cancel, timeout, recovery | Chưa có DAG tổng quát, pause/resume và retry framework |
| Web/API offline | HAR, OpenAPI, inventory, diff, schema observations | JSON Schema chưa toàn bộ; chưa resolve file/network tùy ý |
| HTTP / Identity | Từng request loopback có review, OS secrets, history | Chưa remote replay, cookie jar, stateful workflow, active authz matrix |
| AI | Registry 24 profile, discovery, chat, routing, local plans | Cloud account thật và hệ multi-agent đầy đủ chưa verified |
| Tool Manager | ripgrep và YARA-X portable có manifest kiểm chứng | Các scanner/tool khác còn pending; chưa dependency DAG/package managers |
| MCP | stdio/HTTP thật với fixture, schema, approval, persisted jobs | Tích hợp hãng thứ ba và sandbox OS chưa verified/hoàn chỉnh |
| Evidence / Findings / Reports | Hash, traceability, CVSS, merge, 4 định dạng | Template chủ yếu text-first; bulk/performance còn thiếu |
| Nền tảng | Windows native; Linux Core/CLI Parrot/WSL | Linux Desktop, packages và Secret Service chưa verified |

Network Security, Mobile Security, Cloud & Containers, CI/CD Security và Red Team Lab chưa có workspace chuyên biệt hoàn chỉnh. Mục trong sidebar hoặc tool catalog không phải bằng chứng những domain này đã được triển khai. Phụ lục C dẫn source và trạng thái từng trang.

## 2.3. Yêu cầu phi chức năng

Các yêu cầu nền tảng gồm không làm mất dữ liệu cũ, không lưu key trong JSON/SQLite thông thường, không tự chạy imported content, có lỗi rõ ràng khi vượt giới hạn, không giả mạo progress, và không cho output AI tự trở thành Verified finding. Các điểm đã có test được ghi ở chương kiểm thử; các giới hạn như OS sandbox, encrypted artifact vault, workspace backup/restore hoàn chỉnh vẫn là việc còn thiếu.

# 3. Công nghệ và tổ chức source

## 3.1. Stack thực tế

| Lớp | Công nghệ / cấu hình được khóa |
| --- | --- |
| Ngôn ngữ backend | Rust edition 2024; rust-version 1.95 |
| Shared storage | rusqlite 0.40.2, bundled SQLite, WAL và transaction |
| Async / HTTP | Tokio; reqwest 0.13.5 với TLS verification |
| Desktop bridge | Tauri 2; typed command bridge |
| Frontend | React 19.3.0, TypeScript 7.0.2, Vite 8.3.4 |
| Frontend tests | Vitest 5.0.4, Testing Library, Playwright/native WebView2 harness |
| AI | Các adapter protocol riêng, OS SecretStore, catalog JSON |
| MCP | rmcp 3.5.1 theo Cargo.lock; stdio và Streamable HTTP |
| PDF ứng dụng | Rust/krilla và font local; không cần cloud renderer |
| PDF kiểm thử | pypdf 6.19.0, pin hash; chỉ dùng trong acceptance |

Các số trên là snapshot lockfile/manifests, không phải khuyến nghị tự nâng toàn bộ dependency. Cargo.lock và apps/desktop/package-lock.json cần được giữ đồng bộ với các thay đổi đã test. Toolchain đã dùng trên Windows là Rust 1.95.0 MSVC, Node 24.15.0, npm 11.7.0 cùng Visual Studio C++ Build Tools/Windows SDK/WebView2.

## 3.2. Cấu trúc thư mục

| Đường dẫn | Vai trò |
| --- | --- |
| crates/reconx-security | AppError, validation, gateway và execution policy cơ sở |
| crates/reconx-storage | Kết nối SQLite, schema, migration, preflight và backup nâng cấp |
| crates/reconx-ai | Provider catalog, protocol, discovery, streaming, SecretStore |
| crates/reconx-tools | Manifest, download/verify, portable install và adapter giới hạn |
| crates/reconx-analysis | Static parsing, hash, entropy, strings, giới hạn file |
| crates/reconx-core | Các application service, quan hệ dữ liệu, jobs và audit |
| apps/cli | Binary reconx, Clap, JSON output và tích hợp Core |
| apps/desktop/src | React pages, shared components, state và theme tokens |
| apps/desktop/src-tauri | Native lifecycle, IPC, dialog và job/event integration |
| manifests/tools | Pinned tool distributions, hash và nguồn provenance |
| tests/e2e | Native scenarios, mock AI/MCP, picker automation và fixture |
| scripts / packaging | Build, verification, release smoke, cấu hình NSIS/deb |
| docs | Ledger, kiến trúc và hướng dẫn chuyên đề |
| artifacts | Binary, log, screenshot, test DB và proof local; bị loại khỏi Git |

Frontend không parse binary hoặc tự thực hiện business rule. Những phần tử UI mới nên gọi service typed đang tồn tại. Không thêm scheduler React riêng hoặc dựng dữ liệu giả khi backend trả lỗi/chưa cấu hình.

# 4. Kiến trúc ứng dụng và các ranh giới

## 4.1. Phân lớp và luồng dữ liệu

| Bước | Thành phần | Trách nhiệm |
| --- | --- | --- |
| 1 | React / CLI | Thu thập input, hiển thị trạng thái, yêu cầu review |
| 2 | Tauri bridge / Clap handler | Deserialize typed input và chuyển đến Core |
| 3 | Core service | Validate association, authorization, scope, revision, budget |
| 4 | Job Engine hoặc thao tác quản trị bounded | Claim thực thi, cancellation, timeout, logs |
| 5 | Adapter / parser | Tạo kết quả thực tế; không nhận shell string tùy ý |
| 6 | Storage / artifact store | Transaction metadata, file immutable, hash và evidence |
| 7 | Finding / Report | Người nghiên cứu đánh giá; snapshot redacted có approval |

Desktop sử dụng typed Request enum với tag op, snake_case và deny_unknown_fields. Core là nguồn quy tắc chung cho Desktop/CLI. Tác vụ blocking chạy ngoài UI thread; AI request là async, stream được chuyển qua IPC channel có cancellation. Event phục vụ cập nhật UI nhưng dữ liệu bền vững vẫn là nguồn khôi phục sau restart.

## 4.2. Đồng bộ Desktop và CLI

Hai client không đồng bộ qua cloud. Chúng mở cùng database và managed directories. CLI ưu tiên --database; tiếp theo là RECONX_DATA_DIR/reconx.db; cuối cùng là OS application data directory. Desktop dùng cùng RECONX_DATA_DIR/default. Nếu hai client mở hai path khác nhau, đó là hai workspace khác nhau chứ không phải lỗi dữ liệu tự biến mất.

Job claim được điều phối trong SQLite nên hai process không tự vượt concurrency chung. Khi CLI sửa target hoặc tạo job, Desktop đọc persisted state/event để hiển thị. Lựa chọn project trong giao diện là presentation context, không ngầm thay đổi tham số của một lệnh CLI đang chạy.

## 4.3. Quan hệ dữ liệu khái quát

Project có nhiều Target, ImportedFile, Job, Evidence, Finding và Report. ImportedFile có managed sample, target và nhiều BinaryAnalysis. Job có thể liên kết target/file, MCP invocation, schema review hoặc replay request. Evidence liên kết project/target/job; Finding liên kết nhiều Evidence qua finding_evidence. Report là snapshot dựa trên danh sách finding/evidence đã chọn, không phải view luôn thay đổi theo các bản ghi hiện tại.

Provider profile, agent routing và settings thuộc workspace. HTTP identity gắn project/target/exact origin và opaque secret reference. MCP server lưu public config/revision/discovered schema, còn secret nằm trong OS store. Các association được kiểm tra trong Core trước khi commit; frontend không được coi một ID tồn tại là đủ quyền dùng nó ở project khác.

# 5. Database, migration và bảo toàn dữ liệu

## 5.1. Schema hiện hành

Schema hiện tại là **012**. Migration SQL theo version được chạy tăng dần bằng transaction. 007/008 dành cho các snapshot phát triển bị gián đoạn; source không tái sử dụng các số này hoặc xóa bảng không nhận diện từ workspace cũ. Phụ lục B liệt kê tên migration và bảng thật.

Các mốc chính: 001 foundation; 002 inputs; 003 durable jobs; 004 tool/MCP/capability plans; 005 evidence/findings/reports; 006 local inventory; 009 AI profiles/routing; 010 HTTP imports; 011 schema review binding; 012 identity/draft/manual replay. Những cải tiến dependency/schema constraint/DB preflight gần nhất không tạo migration mới.

## 5.2. Mở workspace và nâng cấp

Source mới kiểm tra schema bằng kết nối SQLite read-only trước khi bật WAL, sau đó kiểm tra lại bên trong write transaction. Database mới hơn build hoặc file hỏng bị từ chối thay vì tự sửa user_version hay viết thêm journal. Test kiểm tra byte identity, journal mode và không phát sinh WAL/SHM trong các tình huống bị từ chối.

Nâng cấp additive có snapshot nhất quán bằng VACUUM INTO trước transaction migration; backup bao gồm dữ liệu committed trong WAL. Nếu tạo backup thất bại thì không tiếp tục migration. Cần dừng các client cũ trước khi nâng cấp. Kiểm tra version sớm không phải bảo đảm chống mọi race khi một process khác đồng thời thay đổi file.

Không downgrade bằng cách chỉnh user_version, xóa table hoặc thay database bằng file rỗng. Khi có lỗi migration, giữ nguyên cả file hiện tại lẫn backup; dùng app tương thích với bản backup và xác minh restore trên bản sao trước khi tiếp tục.

## 5.3. Tổ chức artifact và credential

SQLite chứa metadata, JSON normalized, path/reference có kiểm soát và các quan hệ. Sample, analysis JSON lớn, evidence và report snapshot nằm ngoài DB trong managed directories bên cạnh database. Tên artifact được tạo bằng UUID/hash, không dùng trực tiếp filename/path do input không tin cậy cung cấp.

Artifact không phải encrypted vault. Unix dùng permission riêng cho file/thư mục; Windows thừa kế ACL của workspace. Thư mục tùy biến được chia sẻ phải được reviewer kiểm tra quyền. Đồng bộ một folder qua dịch vụ cloud có thể đưa dữ liệu trong folder đó lên dịch vụ; không đặt workspace chứa original HAR/binary/credential-bearing evidence vào thư mục chia sẻ nếu chưa đánh giá riêng.

API key, bearer, cookie, Basic password và custom secret headers dùng OS SecretStore, không ghi plaintext vào bảng thông thường. Bản sao SQLite không bao gồm những secret này. HTTP identity namespace còn phụ thuộc canonical DB path; di chuyển DB sang path khác cần cấu hình lại credential. Không nên xem việc copy riêng reconx.db là một bản backup đầy đủ.

## 5.4. Recovery và giới hạn

Quy trình backup/restore toàn workspace bằng một nút chưa được triển khai. Nếu backup thủ công, dừng tất cả client, lưu DB và managed artifacts nhất quán, kiểm tra hash, mã hóa bản sao nhạy cảm, và ghi rõ yêu cầu cấu hình lại key qua OS store. Không export key ra file để tiện backup.

Xóa target giữ lịch sử cần thiết bằng nullable foreign key/ON DELETE SET NULL ở những bảng liên quan. Identity bị detach trở thành expired/target_removed, không tự đăng ký lại target hoặc phát lại request. Xóa imported record không ngầm xóa original sample/cache/history; cache garbage collection còn thiếu.

# 6. Project, Target và Scope Manager

## 6.1. Project

Project gom phạm vi, target, jobs, evidence, findings và report. Tạo project với tên/description hợp lệ; tên trùng trả lỗi có mã. Có archive/restore và metadata import/export. Project metadata export không tự chứa toàn bộ target hoặc byte của imported sample; target export và managed-data backup là nghiệp vụ riêng.

## 6.2. Target inventory

Action + Add Target có ở Dashboard, Projects, Targets & Scope và Quick actions. Dialog nhận tên, loại, value, project, description, tags, allowed/excluded scope, authorization và research mode. Inventory có search/filter theo project/type, edit/remove, import/export, detail, scope validation và linked file/job.

17 target kind thực tế là domain, subdomain, url, ipv4, ipv6, cidr, hostname, rest_api, graphql, git_repository, source_directory, binary, shared_library, firmware, apk, container_image, iac_directory. Core normalize/validate theo loại. Local path chưa tồn tại có thể được đăng ký; path hiện có phải phù hợp file/directory category. Domain quốc tế dùng punycode. URL target không nhận embedded credentials/query/fragment.

Tag là grouping hiện có; chưa có hierarchical group manager. Duplicate type/value bị kiểm tra. Target JSON import format_version 1 có 1–500 records, tối đa 1 MiB; import atomic, không để lại một nửa danh sách khi một record sai. Export không overwrite file đích đã tồn tại.

## 6.3. Scope

Scope hỗ trợ exact host, wildcard descendant, HTTP(S) origin/path prefix, IP/CIDR và absolute local path. *.example.test không bao gồm apex example.test. Scheme, effective port và path component boundary có ý nghĩa. Exclusion được ưu tiên hơn allowed. Nếu allowed_scope trống, default là exact target value.

Authorization acknowledgement được lưu riêng với kết quả in-scope. Một candidate có thể in-scope nhưng chưa được authorized. Mọi operation cần thực thi phải kiểm tra lại policy của mình; target registration/scope validation không gửi mạng và không cấp tool execution. Active mode trên app và target phải phù hợp operation.

## 6.4. Ví dụ CLI

```powershell
reconx --json project create demo --description "Local research"
reconx --json target add --project demo --type domain --value example.test --authorized
reconx --json target list --project demo
reconx --json target show TARGET_ID
reconx --json scope show TARGET_ID
reconx --json scope validate TARGET_ID --value api.example.test
reconx target export --project demo --output new-targets.json
```

TARGET_ID và tên file ở các ví dụ là placeholder cần thay, không phải dữ liệu người dùng thật. Lệnh target update nhận TargetInput hoàn chỉnh từ export, không nhận nguyên response có id/timestamp của show. Không dùng domain trong ví dụ để suy ra ứng dụng có quyền active scan trên Internet.

# 7. Reverse Engineering File Workspace

## 7.1. Import và luồng phân tích

Chọn project, mở Reverse Engineering, Import File hoặc Import Folder qua native dialog; xác nhận quyền sử dụng file. Drag-and-drop chuyển path thật cho native service; không dựng mock File record từ browser. Folder import chỉ nhận số entry trực tiếp giới hạn, không recursively giải nén/thực thi nội dung.

Import tạo managed sample có hash và record liên kết target. Có thể import khi chưa authorized nhưng analysis bị chặn cho đến khi policy được xác nhận. Analyze File tạo job thật; UI đọc status/log/result và lịch sử persisted. Có notes cục bộ, search trong kết quả retained và tabs metadata/sections/strings/imports/exports.

## 7.2. Dữ liệu được trích xuất

Parser nhận dạng bằng content, hỗ trợ PE, ELF và thin Mach-O. ZIP/APK có directory metadata đọc không extraction; AndroidManifest.xml giúp nhận dạng APK nhưng không tương đương manifest security analyzer. Format lạ có raw hash/size/entropy/string inspection. Architecture, entry point, sections/segments/imports/exports chỉ hiển thị khi parser thực sự có dữ liệu; trường không hỗ trợ phải là unavailable.

ASCII và UTF-16 LE sampling có giới hạn; không quảng cáo exhaustive strings. Hex view hiện là preview bounded. Binary compare là đối chiếu byte/section/import, không phải semantic patch diff hoặc function-matching. Source sample không bị sửa và không bị thực thi.

| Tài nguyên | Giới hạn hiện có |
| --- | --- |
| Sample mỗi file | 64 MiB |
| Section/import/export/package entry | Tối đa 4.096 mỗi loại được hỗ trợ |
| Strings giữ lại | 4.000; tối đa 256 ký tự/string |
| Folder trực tiếp | 100 entries |
| Hex preview | 512 bytes |
| Entropy section | Shared 64 MiB scan budget, kể cả vùng overlap |

## 7.3. AI và external analysis

AI artifact review chỉ gửi preview được chọn như metadata, imports hoặc strings sample khi người dùng duyệt đúng nội dung và destination. Không tự upload sample hoặc toàn bộ binary. Paths/notes không ngầm được kèm theo; strings có thể chứa dữ liệu riêng nên vẫn phải review. Output là interpretation chưa verified.

YARA-X adapter dùng fixed embedded signatures cho file header/benign fixture marker. Không coi signature match là vulnerability. Ghidra, Rizin, Radare2, Binwalk, Frida, WinDbg và x64dbg chưa có end-to-end adapter được xác nhận trong build này. MCP kết nối thành công đến fixture echo không chứng minh decompilation hoặc debugger integration.

```powershell
reconx reverse import C:\Owned\sample.exe --project demo --authorized
reconx --json reverse list --project demo
reconx --json reverse analyze FILE_ID
reconx --json reverse metadata FILE_ID
reconx --json reverse strings FILE_ID
reconx --json reverse compare FILE_A FILE_B
```

metadata và strings đọc analysis đã lưu; không lặng lẽ tạo job mới. analyze dùng cùng handler/queue với Desktop và chờ kết quả có committed state. Remove giữ nguyên cache bytes và history theo semantics bảo toàn dữ liệu, không phải secure erase.

# 8. Persistent Job Engine

## 8.1. Mô hình và concurrency

Job lưu id, project, target/file, kind, creation/start/end, state, progress, error, result và execution log. Các state là queued, running, completed, failed, cancelled. Phần trăm phản ánh processed bytes/stage thật; không sinh tiến độ bằng timer.

Claim transaction trong SQLite giới hạn tối đa **hai running jobs trên một database**, kể cả nhiều Desktop/CLI process. Cancellation đang chờ được xác nhận vẫn giữ capacity slot. Queued cancellation tức thời; terminal update có worker/state fencing để worker cũ không publish kết quả sau khi mất lease hoặc bị hủy.

Worker heartbeat mỗi 500 ms với lease 10 giây. Startup/poll reconcile expired hoặc legacy ownerless running jobs thành interrupted failure hoặc cancelled khi đã có cancel request. Queued jobs còn sau restart. Interrupted HTTP không được tự gửi lại.

## 8.2. Timeout, cancellation và giới hạn cách ly

Timeout tính từ lúc claim, không gồm thời gian chờ queue; generic local job nhận khoảng 1 ms đến một giờ, handler-specific policy có thể chặt hơn. Các điểm đọc/hash/string/validation bounded kiểm tra cancellation, scope, timeout và shutdown. Parser chạy ngoài UI thread với cardinality limits và panic boundary.

Đây là cooperative cancellation, không phải hard preemption/sandbox của parser trong OS process riêng. Cancel HTTP không thể hoàn tác thay đổi service đã nhận trước đó. Chưa có full DAG, retry/backoff tổng quát, pause/resume hoặc scheduling phân tán nhiều máy.

## 8.3. Job Center và CLI

Quick actions → View Jobs mở danh sách active/completed/failed, filter/search, logs, result và Cancel. Khi UI poll, nó đọc backend state chứ không tạo status giả. Trên CLI, enqueue trả queued record; jobs run chạy foreground để drain queue; jobs run --watch tiếp tục chờ. Không truyền job ID cho jobs run.

```powershell
reconx --json jobs enqueue --kind binary_metadata --file FILE_ID --timeout-ms 60000
reconx jobs run --concurrency 2
reconx --json jobs list --project demo
reconx --json jobs show JOB_ID
reconx --json jobs result JOB_ID
reconx --json jobs cancel JOB_ID
```

Kết quả và completed state được publish trong cùng transaction cùng evidence reference khi áp dụng. Failed/cancelled job không được dựng result evidence như thể đã hoàn thành. Crash có thể để lại orphan file chưa được transaction tham chiếu; garbage collection vẫn là việc cần bổ sung.

# 9. Offline Web/API: HAR và OpenAPI

## 9.1. HAR 1.2 importer

Web Security và API Security chia sẻ HTTP service. Chọn project/target, Import HAR bằng picker hoặc drop, review filename/title và optional captured identity/role label rồi import. Backend validate toàn bộ trước insert để tránh partial import.

Parser lấy request/response method, URL, headers, cookies, body metadata, status, MIME/content type, timings, query/path parameter names và base64 response content được hỗ trợ. Endpoint được nhóm theo origin/method/path; duplicate counts là counts của traffic nhập vào, không phải phát hiện asset mới từ Internet.

Normalized preview mask query values, cookie values, non-allowlisted headers và sensitive JSON fields được nhận diện. Non-JSON hoặc binary body preview có thể bị withheld. Captured identity/role là nhãn người dùng gán cho evidence, không phải xác thực account đã thành công.

## 9.2. OpenAPI importer và endpoint inventory

OpenAPI 3.0/3.1 JSON/YAML cung cấp servers, operations, parameters, request/response schemas, security requirements và components. Same-document JSON Pointer được kiểm tra/expand bounded. Structural cycle không bị expand vô hạn; recursive schema giữ ref cần thiết.

Correlation yêu cầu origin scheme/host/effective port phù hợp. Root-relative servers có thể dùng để browse inventory trên captured origin; schema review chặt hơn cần absolute origin. Concrete paths ưu tiên so với whole-segment parameter template. Server variable default được expand một lần với giới hạn, không biến schema thành execution script.

Target-specific definition không tự áp sang capture của target khác. Project-level definition có phạm vi project được quy định riêng. Target IDs vẫn nằm trong grouping, tránh trộn hai target có endpoint giống nhau. Undocumented endpoint là observation, không tự thành vulnerability.

## 9.3. Resource budgets và original evidence

| Tài nguyên | Giới hạn |
| --- | --- |
| Input regular file | 16 MiB, từ chối final symlink/reparse |
| HAR entry hoặc OpenAPI operation | 2.000/import |
| Decoded body | 256 KiB; encoded source text tối đa 512 KiB |
| Header/cookie/parameter collection | 256 |
| Parsed tree | Depth 48; 200.000 nodes/events |
| References / expansion | 4.096 refs; 32 hops |
| Workspace collection | 5.000 captures/operations; 32 MiB/collection |
| Traffic page | 100 rows; inventory/matrix vẫn đọc bounded collection |
| Redacted traffic export | Tối đa 64 MiB; không âm thầm chỉ xuất trang đầu |

Original HAR/spec được lưu Restricted evidence có SHA-256. Nó có thể chứa token/session; normalized redaction không xóa secret trong original. Raw local export cần explicit approval riêng. Không import secret-bearing HAR lên cloud AI tự động. Multi-file/network ref, YAML alias/anchor/merge/arbitrary tag và full JSON Schema chưa được hỗ trợ.

```powershell
reconx --json web import-har capture.har --project demo --identity alice --role reader
reconx --json api import-openapi openapi.yaml --project demo
reconx --json web traffic --project demo --offset 0
reconx --json web inspect CAPTURE_ID
reconx --json api endpoints --project demo
reconx --json api inspect OPERATION_ID
reconx --json web export --project demo new-redacted-output.json
```

# 10. Schema consistency, captured comparison và observations

## 10.1. Schema review hiện có

Schema check so sánh một HAR capture với operation đã chọn, không gửi request. Source evidence/hash, original HAR JSON Pointer, decoded-body pointer và OpenAPI location đi cùng observation. Core xác minh original artifacts và normalized record, yêu cầu cùng project/exact target association và origin phù hợp trước thực thi.

Coverage có type/required/properties/homogeneous items; nullable 3.0 và null union 3.1; readOnly/writeOnly theo direction trong phạm vi hỗ trợ; status exact/range/default; MIME, required headers/query/cookie/path presence và Basic/Bearer/API key mechanism presence. Có enum, const ở 3.1, numeric/string/count constraints, allOf/anyOf/oneOf/not và additionalProperties boolean/schema.

Branches dùng ba kết quả Matches/Mismatch/Unknown. Unsupported hoặc redacted data không được tính là match thành công. Constraint errors không copy payload/credential vào message. Numeric equality không chỉ ép mọi integer qua f64; Unicode length tính code points trong phạm vi test. Budgets chung áp dụng cả nhánh/ref, không cấp lại ngân sách cho mỗi branch.

| Nhãn observation | Diễn giải |
| --- | --- |
| confirmed_documented_mismatch | Captured data mâu thuẫn một rule cụ thể đã kiểm được |
| possible_mismatch | Có dấu hiệu nhưng chưa đủ khẳng định |
| insufficient_evidence | Thiếu/withheld/redacted dữ liệu để đánh giá |
| unsupported_validation_case | Gặp construct ngoài coverage |

“Confirmed” ở đây là documented schema mismatch, không phải Verified security vulnerability. Không có mismatches không đồng nghĩa toàn bộ schema hợp lệ. Regex/patternProperties, format assertions, uniqueItems, tuples, unevaluated/dependent keywords, custom dialect/resource scope và multi-file refs chưa đầy đủ.

Giới hạn job: depth 32, 10.000 validation steps, tối đa 200 observations và 1 MiB output. Vượt budget trả incomplete coverage rõ ràng. Không có file/network resolver hoặc imported regex/code execution.

## 10.2. Captured role matrix và diff

Offline identity matrix nhóm captures theo origin/method/documented path/identity/role/target. Nó ghi counts, statuses, paths, body hashes và source IDs. api diff so sánh hai response đã có, optional ownership field và lưu comparison evidence.

Đây chưa phải active authorization matrix. 200 OK, body length khác hoặc owner field khác chỉ là observation trong context của capture. Không biết expected business policy thì không được kết luận BOLA/BFLA. Finding tạo từ comparison mặc định cần verification; không có tự promotion bởi AI hay HTTP status.

```powershell
reconx --json api schema-check CAPTURE_ID OPERATION_ID
reconx jobs run
reconx --json jobs result JOB_ID
reconx --json api authz-matrix --project demo
reconx --json api diff CAPTURE_A CAPTURE_B --owner-field owner
reconx --json api finding COMPARISON_ID
```

# 11. Manual HTTP request editor và authentication profiles

## 11.1. Chuỗi thao tác Desktop

Đăng ký authorized URL/REST API/GraphQL target là loopback service do người dùng kiểm soát. Chọn active mode phù hợp trên cả app và target. Trong HTTP workspace, chọn captured request và Edit request / replay to loopback. Captured credentials/query values bị loại khỏi seed; body withheld không được tự sao chép.

Editor hỗ trợ method, URL, public headers, UTF-8 body/content type, timeout và optional identity. Save request chỉ lưu draft. Review send hiển thị exact destination, pinned socket, identity và budget. Send approved request chỉ chạy sau khi đã duyệt fingerprint hiện hành. Thay draft, identity revision, scope hoặc mode làm review cũ mất hiệu lực.

History hiện Job Engine state thật; Cancel request hủy client operation. Open captured response dùng inspector/diff/schema service đang có; Save result evidence dùng Evidence Engine. JSON, URL-encoded form và raw UTF-8 được hỗ trợ trong giới hạn; multipart/binary upload chưa có.

## 11.2. Chính sách destination và budget

| Quy tắc | Hành vi hiện tại |
| --- | --- |
| Destination | Loopback IPv4/IPv6; exact localhost pin 127.0.0.1, không DNS |
| Redirect / proxy | Không follow; không system proxy; response redirect chỉ được capture |
| Approval | Một exact request cho mỗi approval; không tự retry |
| Rate | Tối đa 20 approvals/project/phút; spacing dispatch tối thiểu 500 ms |
| Request / response | Body 64 KiB / 256 KiB, kể cả chunked response |
| Header | Request tối đa 32 public headers; response 64 headers/64 KiB |
| Timeout | 500–10.000 ms; connect timeout 2.000 ms |
| TLS | Certificate validation bật; untrusted local certificate phải bị từ chối |
| State change | POST/PUT/PATCH/DELETE cần Lab Research trên cả app và target |

Các method hiện có GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS không có nghĩa được phép đến mọi host. Other DNS names, mapped IPv6, embedded credentials, URL fragment, out-of-scope path và identity origin sai bị từ chối. Việc không dùng DNS/proxy/redirect trong adapter này giới hạn đường scope escape; không phải bằng chứng đã giải quyết toàn bộ bài toán arbitrary remote networking.

## 11.3. Authentication profile

Authentication manager hỗ trợ Anonymous, Cookie, Bearer, API Key header, Basic và Custom Header. Record chứa role/name/type/project/target/exact origin/revision/expired status; secret lưu trong OS store qua opaque reference. Show/hide chỉ áp dụng input vừa nhập, không hiển thị lại key đã lưu.

Credential replacement tạo immutable reference mới và dùng revision chống stale write. Retired-secret journal cho phép retry cleanup nếu OS store lỗi. auth cleanup báo failure thay vì ngầm bỏ qua. Đây không phải transaction atomic xuyên OS credential service và SQLite.

Saved identity không tự trở thành authenticated. HTTP success cũng không chứng minh quyền truy cập business object. Xóa target giữ history và làm identity target_removed/expired; Test/Edit không tiếp tục dùng target đã xóa. Profile từ project/origin khác không được trộn vào request.

Captured response được redaction trước persistence với secret known/exact encodings và các pattern nhận diện. Body hash/byte count/total elapsed giữ provenance. Capture là application representation đã làm sạch, không phải exact wire transcript. Unknown secret, binary content và arbitrary free text vẫn cần reviewer.

## 11.4. CLI và phần chưa có

```powershell
reconx --json http seed CAPTURE_ID --target TARGET_ID
reconx --json http save request.json
reconx --json http preview DRAFT_ID
reconx --json http replay DRAFT_ID --fingerprint REVIEW_HASH --approve
reconx jobs run
reconx --json http history DRAFT_ID
reconx --json auth profiles --project PROJECT_ID
reconx auth add --project PROJECT_ID --target TARGET_ID --name Alice --role user --kind bearer
reconx auth cleanup
```

Credential được nhập hidden prompt hoặc controlled stdin, không đặt trên command-line. Draft JSON không chứa auth token. Request input có project_id, target_id, source_exchange_id, title, method, url, headers, content_type, body, auth_profile_id, timeout_ms, follow_redirects:false, proxy:null; id/revision/timestamp là output metadata và update revision truyền riêng.

Remote replay, configurable proxy/CA/redirect chain, cookie jar, OAuth refresh, session orchestration, multi-step stateful setup/cleanup, automated BOLA/BFLA và AI HTTP execution chưa được triển khai. Không gọi adapter hiện tại là một công cụ authorization-testing tự động hoàn chỉnh.

# 12. AI Provider & Model Manager

## 12.1. Quy trình cấu hình

Settings → AI Providers → Add AI Provider → chọn provider → chọn preset model → nhập API key → Save. Built-in providers nạp endpoint/protocol mặc định; user thường không phải điền Base URL. Keyless local profile không yêu cầu cloud key khi server không cần authentication. Custom Gateway có Advanced Settings cho endpoint/model/options.

Save là local persistence, hiển thị Saved / Not tested. Test connection gửi một inference request tối thiểu đến đúng model đã chọn; chỉ thành công thật mới cập nhật last-success status/latency. Badge nói về lần test gần nhất, không phải monitor realtime. Authentication, quota, rate limit, timeout, network và inaccessible model là các lỗi riêng; raw error body có thể bị bỏ để bảo vệ secret.

AI Workspace hiển thị enabled configured profiles. Chọn profile khác áp dụng cho future requests ngay, không cần restart. Có nhiều model profile cùng vendor, rename/disable/default/delete và model discovery. Profile thiếu key không được gọi là ready. Phụ lục A mô tả toàn bộ 24 catalog entries từ source.

## 12.2. Protocol, model catalog và discovery

OpenAI Responses, Anthropic Messages và Gemini có adapter protocol riêng; compatible vendor dùng documented interface được ghi trong registry. Không giả định tất cả vendor cùng response schema. Catalog có tên hiển thị, preset ID, official-doc URL, review date và capability metadata; unknown/null không được suy ra là supported.

Refresh Models là explicit request. Discovery có bounded pagination, ID validation, repeated cursor rejection và response limit; selected model không bị tự thay. Model-listing success không chứng minh inference permission/quota của tài khoản. Advanced custom ID hỗ trợ model mới/private/local chưa trong preset. Registry snapshot có thể lỗi thời; actual account access chỉ biết sau authorized live test.

Không tự tải model weights. Ollama/LM Studio/vLLM cần server tương thích đang chạy và model đã cài. Fixture local mock chứng minh protocol/configuration, không chứng minh inference engine hoặc model thật đã có trên máy.

## 12.3. Credential và gateway

Windows sử dụng Credential Manager; Linux dùng Secret Service khi có. Không plaintext fallback. SQLite chỉ giữ public config/model IDs/header names/health/options và secret references. Stored secrets không được trả về UI. Thay key/config làm invalid connection status; concurrent change không được publish stale success.

Endpoint/protocol change khi còn credential bị chặn: phải remove key/custom headers rồi cấu hình explicit destination mới. Custom headers có tên cấm cho host/framing/cookie/proxy authorization. Discovery override là relative path, không phải destination bí mật khác. Custom proxy, timeout, streaming/protocol options nằm trong Advanced Settings và cần review egress.

Headless Linux runner có thể không có Secret Service. Khi xóa provider, service vẫn phải xóa optional key/header đã từng lưu; nếu OS store không sẵn sàng, thao tác fail closed và giữ configuration/routing thay vì tuyên bố xóa thành công. Local tests không được coi case này là Linux credential-runtime verified.

## 12.4. Routing, fallback và privacy

Default profile được dùng khi không có per-agent override. Agent routing hiện là mapping đến saved profiles; nó không chứng minh đã có đủ agent cho mọi domain. Orchestrator/reverse entry points đọc các route có thật, capability checks không được tự giả định model biết tool calling.

Ordinary chat fallback giới hạn theo cấu hình và pre-transmission connection failure; không tự replay HTTP errors/timeouts như thể chưa gửi. Artifact interpretation approval gắn exact preview/destination, không âm thầm theo chat fallback sang provider khác. Proxy-aware artifact approval chưa có nên route này từ chối configured proxy theo policy hiện hành.

Cloud credentials và live provider tests chưa có trong acceptance hiện tại. Các kết quả AI/OpenAI/Anthropic/Gemini fixture phải ghi MOCK_VERIFIED. Không đưa original HAR, session token, binary hoặc target-private data vào prompt khi chưa có informed approval.

```powershell
reconx --json ai catalog
reconx ai providers add --vendor deepseek --name "Research model" --default
reconx --json ai profiles list
reconx ai providers test PROFILE_ID
reconx --json ai models refresh PROFILE_ID
reconx ai models select PROFILE_ID
reconx --json ai routing show
reconx ai routing configure secret-free-routes.json
```

# 13. Capability Registry và AI orchestration

Capability Registry nằm trong Rust, kết hợp local analysis/tool/MCP availability thay vì mỗi React page tự duy trì một danh sách khả năng riêng. Capability cần tool/server/config phù hợp mới được available. Provider profile tồn tại không làm một tool missing tự xuất hiện.

Local-file workflow nhận file metadata, liệt kê capability thật, đề xuất PlanProposal typed, validate allowlist/duplicate/policy, hiển thị review và enqueue existing jobs. Mock model trong integration test tạo kế hoạch nhưng actual local parser/job mới tạo result. Interpretation đọc saved evidence, ghi unverified response riêng, không sửa observed facts.

Không chạy arbitrary model-generated shell commands. Không có đường “AI ra lệnh thì bỏ scope/approval”. MCP tool descriptions và analysis artifacts đều là untrusted input. Kế hoạch có unknown/unavailable/shell/network capability bị từ chối thay vì chuyển thành subprocess string.

Phần còn thiếu gồm full multi-domain Agent Registry, task DAG dependency-aware, pause/resume/checkpoints tổng quát, token/cost budgets, rate-aware scheduling, advanced sharing-aware fallback và Authorized Auto. Không có mười mấy agent đã hoàn chỉnh chỉ vì danh sách role hoặc profile routing tồn tại.

# 14. MCP Hub và trust boundary

## 14.1. Cấu hình và lifecycle

MCP Hub cho đăng ký server stdio hoặc Streamable HTTP. Stdio cần absolute executable có thật, arguments, optional working directory, tên environment secret và explicit trust. HTTP cần endpoint, bearer configuration và timeout. Save không tự biến status thành Connected; Connect handshake/discover thật mới tạo session.

CLI và Desktop dùng chung server configuration/discovered cache/permissions/history trong SQLite. Session connection là process-local: CLI connect/tools thực hiện handshake rồi đóng khi lệnh kết thúc; không đóng được session do Desktop sở hữu. Config edit disconnect local session và invalidate approval; credential/config revision change đòi cấu hình lại key khi cần.

## 14.2. Permission và invocation

Tool Explorer hiển thị discovered schema và policy; input được validate bằng schema bounded. Chỉ tool đã allowlist, explicit approval, readOnlyHint=true, openWorldHint=false, không destructive hint và thuộc class cho phép mới có đường invoke. Exact fingerprint ràng buộc config/tool/schema/input được review.

Invocation qua existing Job Engine, có cancellation/timeout/log/history và short-lived isolated session cho job; không phá session tương tác Hub khi hủy job. Approval được kiểm tra trước kết nối, sau discovery và trước result publication. Result untrusted được bounded/redacted, không tự cấp quyền tiếp theo.

## 14.3. Secrets và isolation thực tế

Secret bearer/named env dùng OS store. Child process khởi tạo từ environment cleared với allowlist runtime ngắn và chỉ secret được khai báo cho server. AI credential interface không được truyền qua MCP API. Known server secret bị echo được redacted cả discovery và kết quả, kể cả escaped forms.

**Stdio không phải OS sandbox.** Process tin cậy vẫn chạy bằng tài khoản người dùng và có thể đọc file/call credential APIs mà tài khoản đó cho phép. Env isolation và tool annotations không chặn được mọi hành vi của code độc. Không có cam kết filesystem/network sandbox, OAuth/SSO hoàn chỉnh hoặc mọi server security tool tương thích.

Discovery hiện một page, tối đa 64 tools/256 KiB; stdio có 4 MiB wire-read ceiling; normalized result/HTTP body 256 KiB; tối đa tám sessions. External schema refs, pagination mở rộng, interactive follow-ups/task execution và arbitrary debugger/network/shell tools không được mở tự động.

```powershell
reconx mcp add --input server.json
reconx --json mcp list
reconx mcp test SERVER_ID
reconx --json mcp tools SERVER_ID
reconx --json mcp inspect SERVER_ID echo
reconx mcp invoke SERVER_ID echo --input input.json --project PROJECT_ID
reconx jobs run
reconx --json mcp history SERVER_ID
```

invoke không có --approve/fingerprint là preview, không được mô tả là đã thực thi. Fixture tests/e2e/mcp-fixture.mjs cung cấp benign echo thật; không phải Burp/Ghidra/Frida/WinDbg/x64dbg integration. Các server hãng thứ ba phải có acceptance riêng với schema thực tế, không điền endpoint/package tưởng tượng.

# 15. Tool Manager và local reconnaissance

## 15.1. Tool lifecycle thực có

Tool Manager có reviewed portable manifest cho ripgrep 15.2.0 và YARA-X 1.21.0. Source/download host, archive hash, extracted executable hash và license/provenance được ghi trong manifests/tools. Installer chỉ lấy named regular entry có giới hạn; không chạy downloaded script hoặc tự elevate.

Installed receipt và health check --version thật quyết định status. PATH executable không tự được adopt/executed. Install có download bytes/cancel trước publication; Repair xử lý managed files rồi cài lại; Remove chỉ xóa named managed files và giữ unexpected user files. Update theo latest reviewed pinned manifest, không tự theo latest upstream chưa review.

Reverse profile gồm ripgrep/YARA-X; Core profile gồm ripgrep. Các profile khác còn pending; profile thiếu integration không được hứa sẽ cài đầy đủ. WSL adapter, apt/dnf/pacman/winget/scoop orchestration, dependency DAG và multi-version rollback chưa hoàn thành. Windows install/capability acceptance có bằng chứng; Linux tool installer runtime chưa được xác nhận bằng các Linux CLI parser tests.

```powershell
reconx --json tools list
reconx --json tools host
reconx --json tools plan yara-x
reconx tools install yara-x
reconx tools verify yara-x
reconx tools install-profile reverse --approve
reconx --json jobs enqueue --kind yara_signatures --file FILE_ID
reconx jobs run
```

## 15.2. Local recon

Recon hiện có reviewed loopback HTTP HEAD inventory. Preview ràng buộc target/policy/input; một request được phê duyệt tạo structured endpoint/header observation, asset và evidence thông qua Job Engine. Không phải vulnerability scanner, crawler hoặc passive Internet intelligence engine.

DNS/RDAP/certificate transparency/history/subdomain enumeration, service scan, broad TLS assessment và adapters Subfinder/HTTPX/DNSX/Katana/Nuclei/Nmap/Amass còn incomplete. Không coi executable được liệt kê trong catalog là adapter usable. Chưa có end-to-end Domain → Full Recon pipeline được chứng nhận.

# 16. Evidence Engine và chain of provenance

## 16.1. Nội dung và metadata

Evidence hỗ trợ HTTP text, JSON, logs, screenshot, binary/static/YARA artifacts, source reference, MCP record, AI summary, note và attachment. Metadata gồm unique ID/project/target/job/kind/title/source/time/sensitivity/review state/SHA-256/size. Original content immutable; review metadata thay đổi riêng.

Completed jobs publish evidence reference cùng committed completion. from-job thu thập completed result cũ theo cách idempotent. Failed/cancelled job không dựng result giả. Từ view/review/export đều kiểm tra integrity trước dùng; hash đúng chỉ chứng minh byte khớp, không chứng minh nội dung đúng hoặc vulnerability đã verified.

## 16.2. Artifact safety

Input symlink/reparse hoặc managed directory bị redirect bị từ chối. Tên source không trở thành filesystem path do user điều khiển. Text entry 1 MiB, file import 32 MiB, text preview 1 MiB; PNG/JPEG preview 8 MiB, tối đa 8.192 pixels/chiều và 16 megapixels kiểm từ header trước decode. Binary/image export cần raw approval vì không có universal image redaction.

observed nghĩa là có real tool result; manually_reviewed cần source-review note; AI summary luôn unverified. Mức sensitivity không tự mã hóa byte trên đĩa. Original HAR/imported attachment có thể chứa secret dù preview sạch. Người chia sẻ phải dùng redacted exports và review, không copy managed original vào báo cáo công khai.

```powershell
reconx --json evidence list --project demo
reconx --json evidence show EVIDENCE_ID
reconx --json evidence verify EVIDENCE_ID
reconx evidence import owned.txt --project demo --kind user_note
reconx evidence from-job JOB_ID
reconx evidence review EVIDENCE_ID --note review.txt
reconx evidence export EVIDENCE_ID --output new-redacted.txt
```

Search/bulk/dedup/pagination lớn và chain-of-custody workflow chuyên sâu chưa hoàn chỉnh. Audit/source/hash/review hiện có là provenance foundation, không phải chứng nhận forensic chain of custody theo một chuẩn pháp lý.

# 17. Findings Manager và CVSS

Finding chứa title/description/target/asset/category/CWE/CVE/severity/confidence/status/technical detail/reproduction/impact/remediation/reference/note/provenance/evidence IDs. Revision chống stale edit giữa Desktop/CLI. Không cho gắn evidence khác project/target một cách tùy ý.

Workflow có Hypothesis, Needs Verification, Verified, Refuted, False Positive, Duplicate, Accepted Risk và Remediated. So sánh hiển thị record gốc; merge giữ duplicate record, chuyển trạng thái Duplicate và hợp evidence links transactionally, không xóa bằng chứng lịch sử.

Promotion Verified cần manual confirmation, verification note và ít nhất một integrity-checked, manually-reviewed, non-AI source. AI confidence, CVSS score, signature match hoặc HTTP status không đáp ứng điều kiện này. CVE/CWE input được validate identifier, không tự tra cứu hoặc bịa mã để tăng vẻ tin cậy.

CVSS 3.1/4.0 dùng Rust cvss library cùng validation độc lập cho required/duplicate/undefined metrics. Partial vector không được nhận default score như complete vector. Severity finding vẫn là lựa chọn đánh giá của researcher, không bắt buộc tự suy từ CVSS khi không có đủ context.

```powershell
reconx --json findings list --project demo
reconx --json findings show FINDING_ID
reconx findings create --input finding.json
reconx findings update FINDING_ID --input finding.json --revision 1
reconx --json findings compare PRIMARY_ID DUPLICATE_ID
reconx findings merge PRIMARY_ID DUPLICATE_ID
reconx findings cvss "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"
```

Vector trên chỉ minh họa syntax scoring, không phải điểm của ReconX AI hay một vulnerability đã phát hiện. CWE taxonomy UI, dedup heuristic và remediation tracking nâng cao vẫn cần phát triển.

# 18. Report Generator và dữ liệu xuất

## 18.1. Snapshot và review

Prepare report nhận project, title/template/summary/methodology và selected finding/evidence IDs. Linked evidence được thu thập và verify. Snapshot immutable giữ scope, tools, technical details, references, verification status và quality warnings. Finding thay đổi về sau không rewrite report lịch sử.

Preview hiển thị redactions và warnings. Export yêu cầu hash của đúng snapshot vừa review; đổi format làm approval UI phải xét lại. Export rechecks evidence integrity và không overwrite file đích. Không chọn finding không đồng nghĩa hệ thống đã assessment và tìm thấy zero vulnerabilities.

## 18.2. Định dạng và redaction

Markdown, JSON, HTML và PDF dùng cùng redacted content. Header/cookie/token/key/private-key/credential-URL/local-path patterns được xử lý; JSON evidence text được redaction cấu trúc trước khi embed. Nested serialized JSON quá giới hạn bị withheld rõ ràng. Unknown secret vẫn có thể lọt qua heuristic nên preview là bắt buộc.

HTML escape content và CSP chặn script/external loads; Markdown escape active markup; JSON giữ structured UTF-8. PDF dùng krilla/font local Source Sans 3, Unicode/subsetting/pagination, không JavaScript/network renderer. Vietnamese/English extraction được kiểm tra bằng parser độc lập. Font của PDF report ứng dụng khác với yêu cầu **Times New Roman** của tài liệu Word tổng thể này; tác vụ viết tài liệu không thay typography của app.

Binary/image raw attachments không âm thầm embed; metadata/hash/warning giữ traceability. Text evidence trên 64 KiB bị omit với warning. Mỗi report tối đa 100 findings/100 evidence và 8 MiB snapshot; PDF có page budget khoảng 400. Template labels hiện chủ yếu cùng text-first layout, chưa có bộ visual report designer hoàn chỉnh.

```powershell
reconx --json report prepare --input report-input.json
reconx --json report show REPORT_ID
reconx report export REPORT_ID --format markdown --output new-report.md --reviewed-sha256 SNAPSHOT_SHA
reconx report export REPORT_ID --format json --output new-report.json --reviewed-sha256 SNAPSHOT_SHA
reconx report export REPORT_ID --format html --output new-report.html --reviewed-sha256 SNAPSHOT_SHA
reconx report export REPORT_ID --format pdf --output new-report.pdf --reviewed-sha256 SNAPSHOT_SHA
```

Chỉ dùng singular **report** và subcommand **prepare** theo CLI hiện tại. Older report snapshot bị phát hiện chưa redaction đúng có thể bị từ chối export với REPORT_REDACTION_REQUIRED; tạo/review snapshot mới, không sửa original evidence để lách kiểm tra.

# 19. Giao diện, typography và ba theme

Shell hiện gồm narrow icon rail, navigation sidebar, toolbar/main workspace và activity/context panel. Mục navigation hiện hữu được giữ; module chưa có workflow thực phải thể hiện trạng thái rõ thay vì cung cấp nút báo thành công giả.

Typography ứng dụng dùng Segoe UI Variable Text trên Windows, Source Sans 3 local fallback, IBM Plex Mono cho technical content; DM Sans giữ cho wordmark. Font không cần Google Fonts runtime. Tài liệu này tuân theo yêu cầu riêng của người dùng: mọi chữ trong DOCX, kể cả code mẫu/bảng/heading/header/footer/mục lục, đều Times New Roman màu đen.

Chỉ có **Light, Dark, ReconX**. Theme manager chung đồng bộ topbar, sidebar và Settings. Theme lưu tức thời qua preference service, khôi phục trước show window để giảm flash. Missing/legacy system preference migrate sang reconx; không có OS appearance listener đổi theme. Light neutral trắng/xám; Dark charcoal; ReconX giữ burgundy/lavender/raspberry identity.

Ctrl+K tìm pages/project/model config, panel có collapse/resize trong phạm vi hiện có. Native acceptance có ba theme ở 1920×1080, 1440×900 và 1366×768 cho luồng identity/HTTP. Kích thước đó là content viewport của renderer; không suy ra mọi tổ hợp display scaling/phần cứng đã test.

# 20. Security Model và dependency assessment

## 20.1. Tài sản và threat boundary

Tài sản nhạy cảm gồm provider key, HTTP identity, original traffic, imported sample, evidence, findings, notes, project metadata và exported reports. Input không tin cậy gồm HAR/OpenAPI/binary/archive/schema, AI output, MCP descriptions/results và tool download. Desktop/CLI phải dùng validation ở Rust; UI checkbox đơn lẻ không đủ làm security boundary.

Threat cần xem xét gồm malicious parser input, path traversal, oversized data, prompt injection, credential echo, out-of-scope request, stale approval, process privileges và interrupted transaction. Các budget/ref/file/command allowlist/revision checks giảm phạm vi nhưng không làm ứng dụng thành sandbox tuyệt đối hoặc hệ DLP tổng quát.

## 20.2. Chín GitHub alerts đã được lấy thật

Tại checkpoint 18/10/2026, tám High nằm ở test-only pypdf 6.16.1; đã pin **6.19.0**, official wheel SHA-256 **7e5d6e730e7dae87d560a2cee218b852f6498c8be61966f3cd02ead971e48d14**. Native independent PDF acceptance đã dùng interpreter cô lập với pin mới. pypdf không nằm trong production Desktop/CLI; PDF ứng dụng do Rust tạo.

Sau push được người dùng cho phép, authenticated Dependabot export xác nhận alerts #2–#9 fixed. Không dismiss, ignore hoặc đổi severity. Các GHSA là qv6h-rv94-w285, 5jq2-8x83-x246, fp3h-c4fm-7vvf, g9cg-prrw-2r8q, jw7q-gvrg-4vj3, w23x-9jrw-r45c, php9-fj8v-98fj, v247-6f48-mgcj.

Một Medium còn mở: **glib 0.18.5, GHSA-wrw7-89jp-8q8g / RUSTSEC-2024-0429**. Linux Desktop dependency path đi qua Tauri/GTK/WebKit. Bug FFI out-pointer trong VariantStrIter có thể vi phạm safety; first patched 0.20 không phải drop-in cho GTK 0.18. Thêm glib mới song song không xóa transitive copy cũ. Source app không gọi trực tiếp không chứng minh unreachable qua dependency.

Dependency gate vẫn **BLOCKED**; không risk waiver. cargo-audit default exit 0 có thể vẫn chứa unsound warning, nên CI dùng --deny unsound. npm audit snapshot có 0 findings. RustSec còn sáu maintenance warnings: proc-macro-error 1.0.4, rustybuzz 0.20.1, ttf-parser 0.25.1, zbus_names 4.3.4, zvariant 5.15.0 và zvariant_derive 5.15.0. Đây không phải sáu exploit High mới được xác lập nhưng cần upstream/path review.

## 20.3. Credential incident và release risk

Một Git credential từng xuất hiện trong diagnostic remote URL; người dùng xử lý revocation riêng. Việc scan text artifacts không thấy pattern không chứng minh token cũ đã revoke hoặc mọi bản sao/history sạch. Không in credential-bearing remote, không copy token vào docs, screenshot hay logs. Push dùng clean repository URL và credential manager; release publication là hành động riêng.

Custom Windows ACL, OS process sandbox, encrypted evidence vault, secure complete backup/restore, temporary/orphan cleanup và independent Linux Secret Service acceptance còn thiếu. Các điểm đó cần hiện trong release checklist, không được xóa khỏi báo cáo vì regression test pass.

# 21. Build, chạy ứng dụng và hướng dẫn phát triển

## 21.1. Windows development

Prerequisites đã dùng: Rust/MSVC 1.95.0, Node 24.15.0/npm 11.7.0, Visual Studio 2022 C++ Build Tools, Windows SDK và WebView2. Không cần admin chỉ để build nếu các prerequisites đã cài. Không tự chạy installer/tool profile lạ khi mở app lần đầu.

```powershell
Set-Location D:\ReconXAI
cargo build --locked -p reconx
Set-Location apps\desktop
npm ci --registry https://registry.npmjs.org
npm run tauri -- dev
```

Custom-protocol build embed frontend nên không cần Vite server khi chạy binary. Cần build frontend trước native executable để tránh chụp screenshot một UI cũ còn nhúng trong binary.

```powershell
Set-Location D:\ReconXAI\apps\desktop
npm run build
Set-Location D:\ReconXAI
$env:CARGO_INCREMENTAL='0'
cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol
.\target\release\reconx-desktop.exe
.\target\release\reconx.exe --help
```

scripts/build.ps1 tạo optimized pair/hash/notices; -Installer gọi Tauri NSIS config per-user. Binary cần đúng frontend asset/hash để so với native evidence. Không copy binary từ lần build khác vào directory có manifest cũ mà không cập nhật proof.

## 21.2. Chạy workspace riêng

```powershell
$env:RECONX_DATA_DIR='D:\Private\ReconX-Demo'
.\target\release\reconx.exe --json project list
.\target\release\reconx-desktop.exe
```

Thư mục ví dụ phải là nơi người dùng chủ động chọn và có quyền phù hợp. Không trỏ test suite vào DB thật để “test nhanh”. --database là explicit override phía CLI; bảo đảm Desktop cùng RECONX_DATA_DIR nếu muốn kiểm tra shared state.

## 21.3. Linux

Optimized CLI đã build/test trên Parrot chạy trong WSL2 x86_64. GUI SDK pkg-config cho GLib/GTK3/WebKit2GTK 4.1 chưa đủ tại local environment. Có WSLg display không tự làm Tauri GUI build/runtime pass. Secret Service, distro packages, clean install và GUI acceptance là gate khác.

CI Ubuntu cài build dependencies trong disposable runner. Điều đó không cho phép suy rằng package đã install hoặc GUI đã chạy ở máy người dùng. Packaging/tauri.linux.json hiện target deb, chưa có Debian clean-install acceptance hoặc AppImage/RPM đã verified.

## 21.4. Thêm tính năng đúng lớp

Thêm model/service trong Rust Core hoặc crate chuyên trách, dùng transaction/ref/index hiện có. Nếu cần migration, dùng version mới, additive và test upgrade từ snapshot cũ; không sửa lịch sử để làm mất bảng. Sau đó expose typed IPC/CLI, frontend state/loading/error/empty, tests và runtime acceptance cùng fixture.

Tool/AI workflow không nhận shell string tùy ý. Secret phải qua SecretStore, evidence phải giữ provenance, result không được tự Verified. Cập nhật ledger/source references/test evidence trong cùng thay đổi. Không tạo hàng loạt placeholder để tăng module count.

# 22. Kiểm thử, CI và bằng chứng thực thi

## 22.1. Baseline local đã kiểm chứng

| Nhóm | Kết quả ghi nhận | Phạm vi |
| --- | --- | --- |
| Windows Rust | 143 passed, 0 failed, 1 ignored | Workspace local tests |
| Linux Rust | 143 passed, 0 failed, 1 ignored | Parrot/WSL Core+CLI; không Desktop |
| Frontend | 26 passed; TypeScript/Vite pass | Components/state/form/regression hiện có |
| Python dependency readiness | 4 passed | Phân biệt source patch/remote closure/unsound/failure |
| Windows native | 9 suites, 77 checkpoints | Actual Tauri/WebView2 + shared CLI fixtures |
| Linux optimized CLI smoke | Passed | Real ELF → job → evidence → finding fixture → 4 report formats |
| Independent PDF | Passed | pypdf 6.19.0; Unicode và redaction cases |
| TLS negative | Passed Windows/Linux CLI | Self-signed local cert bị từ chối; trust store không đổi |
| NSIS | Build/hash pass | Chưa clean install/upgrade/uninstall; chưa signing |

Các số trên được liên kết với artifacts/security-current-verification.json và machine test ledger. Lượt follow-up chỉnh điều kiện test CI được chạy lại local Windows/Linux cùng tổng 143; không tuyên bố tạo thêm production feature từ thay đổi test.

Native suites lần lượt: HTTP replay 11, HTTP workspace 12, shell/settings/native 8, research inputs 9, jobs/recovery 8, tool/MCP/local AI integrations 6, outputs 6, local recon 7, AI providers 10. Screenshots/result JSON giữ theo từng run, không thay ảnh mockup vào proof.

## 22.2. CI từ xa và sự khác nhau với local

Remote CodeQL run 39125535750 cho source c22f44c đã success. Workflow verification run 39125535775 được GitHub chấp nhận YAML, chạy thực sự rồi failure: dependency-security gặp glib đã biết; Linux test giả định Secret Service luôn sẵn sàng; Windows cancellation fixture sử dụng budget quá ngắn để quan sát call đang chạy.

Follow-up test tách cancellation window khỏi timeout scenario và kiểm fail-closed khi OS store không sẵn sàng, giữ configuration/routing. Không nới policy timeout của production để làm test pass. Cần một remote run trên revision mới để xác nhận các test fix; không suy success từ local. Dependency gate vẫn phải đỏ cho glib nếu chưa được giải quyết.

Lỗi YAML trước đó cũng được giữ provenance: plain run chứa --only-binary=:all: không quote làm parser hiểu sai dấu colon. Regression parse cả ci.yml/codeql.yml đã reproduce rồi pass sau folded run string. Syntax acceptance, unit pass, native pass và release pass là các mức khác nhau.

## 22.3. Lệnh kiểm tra

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings
cargo test --locked --workspace -j2
python -m unittest discover -s scripts -p test_dependency_readiness.py -v
Set-Location D:\ReconXAI\apps\desktop
npm run build
npm test
```

Native chạy với explicit RECONX_DESKTOP_EXE/RECONX_CLI_EXE trỏ cùng immutable pair. scripts/verify.ps1 có -Native, -Inputs, -Jobs, -Integrations, -Outputs, -HttpWorkspace, -HttpReplay và -AiProviders tương ứng script hiện có. TLS fixture cần OpenSSL và tự xóa key/certificate tạm, không thêm trust root vào máy.

Test database, synthetic identities và local fixture độc lập với dữ liệu bình thường. Failed log được giữ; chỉ count current run có command/exit/time/evidence phù hợp. Không cộng các lần retry thành số test case duy nhất và không tính ignored là passed.

## 22.4. Coverage và việc QA chưa xong

Có deterministic tests cho validation/duplicates, migration, restart, job failure/cancel/leases, corrupted artifact, malformed/oversized HAR/schema, origin isolation, stale approval, credential echo/redaction, redirect rejection, TLS distrust và output formats. Điều đó không tương đương một penetration test độc lập toàn bộ ứng dụng.

Chưa có release acceptance đủ cả arbitrary remote networking, autonomous authz, every paid provider, third-party debugger MCP, Android/iOS domain, cloud/container analyzers, clean Linux GUI/Secret Service và installer upgrade/uninstall. Các case chưa chạy vẫn phải UNVERIFIED/BLOCKED, không gán pass vì có UI entry.

# 23. Artifact, vận hành và xử lý sự cố

## 23.1. Build candidate có thật

Windows candidate nằm tại artifacts/windows-security-20261018-183806, gồm reconx.exe, reconx-desktop.exe, installer/ReconX AI_0.1.0_x64-setup.exe, SHA256SUMS.txt, THIRD_PARTY_NOTICES.txt và build-manifest.json. Đây là candidate nội bộ, không phải v1.0 installer-certified.

Linux CLI candidate tại artifacts/linux-cli-security-20261018-185625/reconx, kèm hash/compiler/dynamic-library manifest và smoke/acceptance.json. Nó là Linux CLI đã chạy trên Parrot/WSL, không có Linux Desktop/package verified bên cạnh để thay thế bằng file placeholder.

| Artifact | SHA-256 đã ghi nhận |
| --- | --- |
| Windows CLI | 6c43dfa5d119418914fa6f0951aafae33bf7086d6f16e997f89baaa6f9ceba47 |
| Windows Desktop | abec8aac7a2ae746602d211271ed912d4c27f755e2f68a6dffbbbc08915dd6bb |
| Windows NSIS | 6547741712f328b05e13df7364f7aabf83a35b91b35138b68032553ee52fc5f1 |
| Linux CLI | 4511f5eb140496789d7613910ea5f59af3f7128fbe62f1b94051d80cae1b0de9 |

Manifest xác định release_ready:false và clean_install_verified:false. Windows candidate chưa ký. Không có signing infrastructure đã xác nhận; cần certificate/quy trình riêng. Tài liệu không hướng dẫn bỏ qua cảnh báo hệ điều hành để biến installer thành release verified.

## 23.2. Các lỗi thường gặp

| Triệu chứng | Cách kiểm tra phù hợp |
| --- | --- |
| CLI không thấy project của Desktop | So sánh --database, RECONX_DATA_DIR và canonical path; không init/reset DB |
| SECRET_STORE_UNAVAILABLE | Kiểm tra Windows Credential Manager/Linux session Secret Service; không plaintext fallback |
| MODEL_UNAVAILABLE / auth/quota lỗi | Kiểm selected profile/model, saved key và Test connection; không in key/log response thô |
| APPROVAL_STALE | Load draft/policy/schema/identity revision mới rồi review lại |
| Dispatch quá nhanh | Tôn trọng spacing 500 ms và approvals budget; không silent retry |
| Target removed / expired identity | Kiểm target còn liên kết; không replay history bằng target khác |
| Integrity mismatch | Giữ artifact nghi ngờ; điều tra nguồn/backup, không cập nhật hash để lách |
| Schema unsupported | Ghi incomplete coverage; không kết luận schema valid |
| Future/corrupt database | Dừng client, giữ file/backup, dùng phiên bản phù hợp; không sửa user_version |
| UI cũ sau build | Build frontend rồi custom-protocol native; kiểm đúng executable hash |

## 23.3. Dung lượng và housekeeping

Rust target/debug/release/incremental, Node dependencies, immutable binary copies, browser profiles và native artifacts có thể chiếm nhiều dung lượng. Scripts build/verify tắt incremental trong lúc chạy để tránh giữ thêm hàng chục GiB. Cache là dữ liệu tái tạo được nhưng phải phân biệt với imports/evidence/reports/user DB.

Trước dọn, kiểm size và resolved path của từng directory, bảo đảm không có build đang dùng. Chỉ xóa artifact/cache đã xác định có thể tái tạo; giữ test proof hoặc archive một bộ proof có hash trước khi dọn. Không chạy recursive delete theo path chưa kiểm tra hoặc lẫn với data directory. Không xóa user DB/evidence để làm test pass.

## 23.4. Quy trình incident cơ bản

Khi nghi có secret trong log/export: dừng chia sẻ file, giữ evidence của sự cố trong vùng private, xác định phạm vi, nhờ chủ tài khoản revoke/rotate theo provider và tạo lại redacted artifact. Không đăng key vào issue, screenshot hoặc báo cáo. Source cleanup không chứng minh credential đã mất hiệu lực.

Khi nghi imported artifact độc: không thực thi; giữ hash/metadata, dùng isolated environment đã được đánh giá. Parser chạy Rust thread không bảo đảm chống mọi bug native dependency. MCP/tool process dưới cùng account cần trust riêng, không dựa vào label read-only để thay sandbox.

# 24. Release checklist, phần còn thiếu và bàn giao

## 24.1. Điều kiện chặn phát hành

| Gate | Trạng thái cần giữ |
| --- | --- |
| glib unsound advisory | BLOCKED; chưa compatible reviewed fix + Linux GUI validation |
| Sáu maintenance warnings | Cần upstream/path/replacement review |
| Web/API đầy đủ | PARTIAL; remote/session/stateful/active matrix còn thiếu |
| Recon / scanners / agents | PARTIAL; không quảng cáo full autonomous platform |
| Reverse external tooling | PARTIAL; Ghidra/debugger/decompiler cần actual integration |
| Network/Mobile/Cloud/CI-CD/Lab | NOT_STARTED cho dedicated workspace |
| Live cloud interoperability | UNVERIFIED/BLOCKED khi chưa approved accounts |
| Linux Desktop / Secret Service | BLOCKED/UNVERIFIED, không thay bằng WSL CLI |
| Installer clean/upgrade/remove | UNVERIFIED Windows và Linux |
| Signing | BLOCKED khi chưa có infrastructure |
| Backup/restore, ACL, sandbox, large data | Còn hardening và product work |

## 24.2. Thứ tự tiếp tục an toàn và có thể kiểm chứng

Giải quyết compatible glib/GTK safety bằng upstream hoặc backport có provenance/review và Linux SDK/runtime acceptance riêng; không ignore advisory để CI xanh. Trong khi còn block, tiếp tục coverage offline schema, data integrity, manual evidence-linked policy annotations và các defensive/static workflow có fixture xác định.

Từng vertical slice cần backend/DB nếu cần, Desktop/CLI, errors, tests và runtime proof trước khi đánh dấu hoàn thành. Domain chưa có parser/adapter không nên tạo agent label hoặc fake progress. Linux GUI/Secret Service cần graphical/session environment thật; installer phải test ở môi trường clean/upgrade/remove độc lập, không trên user workspace chỉ để lấy screenshot.

## 24.3. Source control và handoff

Không reset, discard, rewrite history, force-push hoặc upload user DB/binary/evidence/secret. Người dùng đã cho phép commit/push source; điều đó không tự cho phép public release, publish dataset hoặc paid/cloud data transfer. Dùng clean repository URL và credential manager, không đọc/in token-bearing remote.

Checkpoint phải ghi HEAD, changed files, migration version, exact commands, platform, exit status, timestamp, evidence path/hash, failed runs còn tồn tại, release blockers và next action. MASTER_EXECUTION_PROGRESS.md là sổ tiếp tục; entries cũ giữ provenance, newest verified checkpoint được ưu tiên khi lịch sử có số test/claims cũ.

## 24.4. Nguồn đối chiếu trong repository

- docs/MASTER_EXECUTION_PROGRESS.md, IMPLEMENTATION_STATUS.md, NEXT_MILESTONE.md, TEST_RESULTS.md và ROADMAP.md.
- docs/RELEASE_READINESS.md, DEPENDENCY_SECURITY.md, release-gaps.json và module-status.json.
- docs/ARCHITECTURE.md, SECURITY_MODEL.md, RESEARCH_INPUTS.md, JOB_ENGINE.md, RESEARCH_OUTPUTS.md.
- docs/WEB_API_WORKSPACE.md, HTTP_SCHEMA_ANALYSIS.md, HTTP_REPLAY.md.
- docs/AI_PROVIDER_MANAGER.md, AI_TOOL_ORCHESTRATION.md, TOOL_MANAGER.md, MCP_HUB.md và THEMES.md.
- artifacts/security-current-verification.json, security-test-ledger.json, documentation-* logs và source-push-result.json.

## 24.5. Tài liệu chuẩn và nguồn upstream

Các URL dưới là nguồn tham khảo chuẩn. Chúng không thay thế proof rằng đúng implementation/account/platform đã chạy. Registry provider giữ official URL riêng trong Phụ lục A; muốn cập nhật model/API phải kiểm chứng lại ở thời điểm thay đổi.

- Tauri prerequisites và IPC: https://v2.tauri.app/start/prerequisites/ ; https://v2.tauri.app/develop/calling-rust/
- HAR 1.2: https://webperfwg.org/specs/HAR/Overview.html
- OpenAPI 3.0.4 và 3.1.1: https://spec.openapis.org/oas/v3.0.4.html ; https://spec.openapis.org/oas/v3.1.1.html
- JSON Schema 2020-12: https://json-schema.org/draft/2020-12/json-schema-validation
- MCP Rust SDK: https://github.com/modelcontextprotocol/rust-sdk
- FIRST CVSS: https://www.first.org/cvss/v3.1/specification-document ; https://www.first.org/cvss/v4.0/specification-document
- RustSec advisory còn mở: https://rustsec.org/advisories/RUSTSEC-2024-0429.html
- pypdf fixed pin: https://github.com/py-pdf/pypdf/releases/tag/6.19.0

# 25. Thuật ngữ

| Thuật ngữ | Ý nghĩa |
| --- | --- |
| Target | Đối tượng được đăng ký; không tự có quyền chạy analyzer |
| Scope | Quy tắc cho phép/loại trừ candidate cụ thể |
| Capability | Backend operation typed thực có, kèm availability/policy |
| Job | Bản ghi thực thi persistent có state/log/result |
| Artifact | Byte file có quản lý và integrity hash |
| Evidence | Artifact/result cộng metadata nguồn gốc và quan hệ |
| Observation | Điều quan sát được; chưa mặc định là lỗ hổng |
| Finding | Đánh giá nghiên cứu có severity/status/evidence |
| Verified finding | Finding đạt manual review và evidence gate cụ thể |
| Fingerprint | Hash của input/config/policy dùng ràng buộc approval |
| HAR | HTTP Archive ghi traffic; import không tự replay |
| OpenAPI | Định nghĩa API; không phải mã được thực thi |
| BOLA/BFLA | Object/function authorization categories, cần business expectation thật |
| MCP | Giao thức khám phá/gọi tool; không tự là sandbox |
| Mock | Fixture xác định dùng kiểm thử, không phải production result |
| Release gate | Điều kiện độc lập phải đạt trước khi gọi build release-ready |
