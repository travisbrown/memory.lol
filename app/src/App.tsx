import { BrowserRouter, Route, Routes } from "react-router-dom";
import { Navbar, Section } from "react-bulma-components";

import "bulma/css/bulma.min.css";
import "./App.css";
import { LoginStatus } from "./LoginStatus";
import { PostLoginRedirect } from "./PostLoginRedirect";
import { TwSearch } from "./TwSearch";

function App() {
  return (
    <BrowserRouter basename="/app">
      <PostLoginRedirect />
      <Navbar aria-label="main navigation">
        <Navbar.Brand>
          <Navbar.Item href="https://memory.lol/">memory.lol</Navbar.Item>
        </Navbar.Brand>
        {/*<Navbar.Container align='left'>
            <Navbar.Item active>home</Navbar.Item>
            <Navbar.Item>documentation</Navbar.Item>
            <Navbar.Dropdown>
              <Navbar.Item href="/about">About</Navbar.Item>
              <Navbar.Divider />
              <Navbar.Item href="/report">Report an issue</Navbar.Item>
            </Navbar.Dropdown>
        </Navbar.Container>*/}
        <Navbar.Container align="right">
          <LoginStatus />
        </Navbar.Container>
      </Navbar>
      <Section>
        <Routes>
          <Route path="/" />
          <Route path="/tw/id/:userId" element={<TwSearch />} />
          <Route path="/tw/:screenName" element={<TwSearch />} />
        </Routes>
      </Section>
    </BrowserRouter>
  );
}

export default App;
