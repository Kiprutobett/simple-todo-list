const BASE_URL ="http://localhost:3000";

async function request(path,options={}) {
    const res =await fetch(`${BASE_URL}${path}`,{...options,credentials:"include",headers:{"Content-Type":"application/json",...options.headers},});
    console.log(res)

    if (!res.ok) {
        const text = await res.text();
        throw new Error(text||`Request failed: ${res.status}`);
        
    }

    if (res.status === 204) return null;

    return res.json().catch(()=>null);
}

export const api = {
    register:(username,password)=> request("/register",{method:"POST",body:JSON.stringify({username,password})}),

    login: (username, password) =>request("/login", { method: "POST", body: JSON.stringify({ username, password }) }),

    me: () => request("/me"),

    listTodos: () => request("/todos"),

    createTodo: (title) =>request("/todos", { method: "POST", body: JSON.stringify({ title }) }),

    updateTodo: (id, changes) =>request(`/todos/${id}`, { method: "PATCH", body: JSON.stringify(changes) }),

    deleteTodo: (id) => request(`/todos/${id}`, { method: "DELETE" }),
}