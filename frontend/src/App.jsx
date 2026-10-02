import { useState ,useEffect} from 'react';
import { api } from "./api";
import AuthForm from "./AuthForm";
import  TodoList  from "./TodoList";
import "./App.css"

function App() {
  const [loggedIn, setLoggedIn] = useState(null)

  useEffect(()=>{
    api.me().then(()=>setLoggedIn(true)).catch(()=>setLoggedIn(false))
  },[]);

  if (loggedIn === null) {
     return <div className='loading-screen'>loading ...</div>
  }

  return (
    <div className='ledger'> 
      
    <div className='ledger-card'>
      {loggedIn?<TodoList/>:<AuthForm onLoggedIn={()=>{setLoggedIn(true)}}/>}
    </div>
    </div>

   
  
  )
}

export default App
