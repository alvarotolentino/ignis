import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import MainMenu from "./pages/MainMenu";
import PlayerSelect from "./pages/PlayerSelect";
import Settings from "./pages/Settings";
import GameView from "./pages/GameView";

function App() {
  return (
    <BrowserRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<MainMenu />} />
          <Route path="/players" element={<PlayerSelect />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/play" element={<GameView />} />
        </Routes>
      </Layout>
    </BrowserRouter>
  );
}

export default App;
