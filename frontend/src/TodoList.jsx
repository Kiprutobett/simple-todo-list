import {useEffect,useState  } from "react";
import { api } from "./api";

export default function TodoList() {
    const [todos,setTodos]= useState([])
    const [newTitle,setNewTitle]=useState("")
    const  [error,setError]=useState(null)

    async function refresh() {
        try {
            setTodos(await api.listTodos())
            
        } catch (err) {
            setError(err.message)
        }
    }

    useEffect(()=>{
        refresh()
    },[]);

    async function handleAdd(e) {
        e.preventDefault();
        if (!newTitle.trim()) {
            return
        }
        await api.createTodo(newTitle);
        setNewTitle("");
        refresh();
    }

    async function toggleComplete(todo) {
        await api.updateTodo(todo.id,{completed:!todo.completed});
        refresh();
    }

    async function remove(id) {
        await api.deleteTodo(id);
        refresh()
    }

     return (
    <div>
      <h1 className="ledger-title">Todos</h1>
      <p className="ledger-subtitle">a running list</p>
      <hr className="ledger-rule" />
 
      {error && <p className="form-message">{error}</p>}
 
      <ul className="todo-rows">
        {todos.length === 0 && !error && (
          <li className="empty-row">nothing on the list yet</li>
        )}
        {todos.map((todo) => (
          <li key={todo.id} className="todo-row">
            <input
              type="checkbox"
              className="todo-check"
              checked={todo.completed}
              onChange={() => toggleComplete(todo)}
              id={`todo-${todo.id}`}
            />
            <label
              htmlFor={`todo-${todo.id}`}
              className={`todo-label ${todo.completed ? "done" : ""}`}
            >
              {todo.title}
            </label>
            <button className="todo-delete" onClick={() => remove(todo.id)}>
              delete
            </button>
          </li>
        ))}
      </ul>
 
      <form className="add-row" onSubmit={handleAdd}>
        <input
          className="add-input"
          value={newTitle}
          onChange={(e) => setNewTitle(e.target.value)}
          placeholder="add a line..."
        />
        <button type="submit" className="btn">Add</button>
      </form>
    </div>
  );


}