import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import heartPulseSvg from "./assets/HeartPulse20Regular.svg";
import "./App.css";

// 自定义 hook
function useInterval(callback, delay) {
  const savedCallback = useRef();

  useEffect(() => {
    savedCallback.current = callback;
  });

  useEffect(() => {
    function tick() {
      savedCallback.current();
    }

    let id = setInterval(tick, delay);
    return () => clearInterval(id);
  }, [delay]);
}

function App() {
  const [heartbeat, setHeartbeat] = useState("")

  useInterval(async () => { // <= useInterval hook
    setHeartbeat(await invoke("heartbeat"));
  }, 1000);

  return (
    <div className="container">
      {
        heartbeat ? (
          <div className="box">
            <i className="icon ico-heartbeat">
              <img src={heartPulseSvg} alt="" srcset="" />
            </i>
            <span>{heartbeat}</span>
          </div>
        ) : (
          <div className="box error">Receiver is not running, please check or reconnect.</div>
        )
      }
    </div>
  );
}

export default App;
