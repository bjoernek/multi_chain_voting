import Header from "./components/header/Header";
import GitHubIcon from "./components/GitHubIcon";

import Voting from "./components/Voting"

function App() {
  return (
    <div className="flex flex-col items-center w-full">
      <Header />
      <div className="flex flex-col items-center w-full gap-10 p-5">
        <div className="h-5 md:h-28" />
        <Voting />
        <GitHubIcon />
      </div>
    </div>
  );
}

export default App;
