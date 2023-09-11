namespace rs volo.example

struct Item {
    1: required i64 id,
    2: required string title,
    3: required string content,

    10: optional map<string, string> extra,
}

struct GetItemRequest {
    1: required i64 id,
}

struct GetItemResponse {
    1: required Item item,
}

struct SetItemRequest {
    1: required i64 id,
    2: required string title,
    3: required string content,
}

struct SetItemResponse {
    1: required Item item,
}

struct DelItemRequest {
    1: required i64 id,
}

struct DelItemResponse {
    1: required bool del
}

struct PingRequest {
    
}

struct PingResponse {
    1: required bool ping
}

service ItemService {
    GetItemResponse GetItem (1: GetItemRequest req),
    SetItemResponse SetItem (1: SetItemRequest req),
    DelItemResponse DelItem (1: DelItemRequest del),
    PingResponse Ping (1: PingRequest ping),
}
