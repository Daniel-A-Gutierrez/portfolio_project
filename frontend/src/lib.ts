enum HTTPMethod
{
    POST="POST",
    PUT="PUT",
    GET="GET",
    DELETE="DELETE"
}

async function fetchJson<I,O>(url: string, method : HTTPMethod, data: I): Promise<O> {
    const response = await fetch(url, {
        method: method,
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(data)
    });

    if (!response.ok) {
        throw new Error(`${response.statusText}`);
    }
    
    return await response.json();
}

///fetch json but when the server's response is nothing.
async function postJson<I>(url: string, method : HTTPMethod, data: I) {
    const response = await fetch(url, {
        method: method,
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(data)
    });

    if (!response.ok) {
        throw new Error(`${response.statusText}`);
    }
}

type KVRow = { id : number, unitxtime : number, key : string, value : string}

//could use zod verification
async function KVGet<T>(key : string) : Promise<T | null>
{
    let res : null | KVRow = await fetchJson("/api/kv_get", HTTPMethod.POST, {key : key});
    return (res == null ? res : JSON.parse(res.value));
}

async function KVGetRow<T>(key : string) : Promise<KVRow | null>
{
    return await fetchJson("/api/kv_get", HTTPMethod.POST, {key : key});
}

async function KVSet<T>(key : string, value : T)
{
    await postJson("/api/kv_set", HTTPMethod.PUT, {key, value : JSON.stringify(value)}); 
}

///requires the existing item to be iterable
async function KVAppend<T>(key : string, value : T)
{
    await postJson("/api/kv_json_append", HTTPMethod.POST, {key, value:JSON.stringify(value)}); 
}

async function KVDelete<T>(key : string, value : T)
{
    await postJson("/api/kv_delete", HTTPMethod.DELETE, {key}); 
}

export {KVGet,KVSet,KVAppend,KVDelete,KVGetRow}
export type {KVRow}