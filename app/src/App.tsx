import { BrowserRouter, Route, Routes } from "react-router-dom";
import {
  Container,
  Content,
  Footer,
  Heading,
  Navbar,
  Section,
  Tile,
} from "react-bulma-components";

import "./App.css";
import { LoginStatus } from "./LoginStatus";
import { PostLoginRedirect } from "./PostLoginRedirect";
import { TwForm } from "./TwForm";
import { TwSearch } from "./TwSearch";
import React from "react";

function App() {
  return (
    <BrowserRouter basename="/app">
      <PostLoginRedirect />
      <Navbar aria-label="main navigation" px="6">
        <Navbar.Brand>
          <Navbar.Item href="https://memory.lol/">
            <img alt="memory.lol" src="/app/logos/dumpster.svg" />
          </Navbar.Item>
        </Navbar.Brand>
        <Navbar.Container align="left">
          <Navbar.Item href="/app/" active>
            Home
          </Navbar.Item>
          <Navbar.Item href="https://github.com/travisbrown/memory.lol">
            About
          </Navbar.Item>
        </Navbar.Container>
        <Navbar.Container align="right">
          <LoginStatus />
        </Navbar.Container>
      </Navbar>
      <Section>
        <Tile kind="ancestor">
          <Tile kind="parent">
            <Tile kind="child" renderAs="article" px="4" pb="4">
              <Heading subtitle>Welcome to memory.lol</Heading>
              <Content pb="4">
                <p>
                  This site is an instance of software from the{" "}
                  <a href="https://github.com/travisbrown/hassreden-tracker">
                    Hassreden-Tracker{" "}
                  </a>
                  project. Search results are limited for unauthenticated users
                  (see{" "}
                  <a href="https://github.com/travisbrown/memory.lol#current-access-restrictions">
                    this document
                  </a>{" "}
                  for details).
                </p>
                <p>
                  Please contact us{" "}
                  <a href="mailto:travisrobertbrown@protonmail.com">by email</a>{" "}
                  to discuss trusted access.
                </p>
              </Content>
            </Tile>
            <Tile kind="child" px="4" pb="4">
              <Heading subtitle>Twitter history search</Heading>
              <Container>
                <TwForm />
              </Container>
            </Tile>
          </Tile>
        </Tile>
        <Routes>
          <Route path="/" />
          <Route path="/tw/id/:userId" element={<TwSearch />} />
          <Route path="/tw/:screenName" element={<TwSearch />} />
        </Routes>
      </Section>
      <Footer>
        <Container>
          <Content style={{ textAlign: "center" }}>
            <p>
              <a href="https://github.com/travisbrown/hassreden-tracker">
                <strong>Hassreden-Tracker</strong>
              </a>{" "}
              and{" "}
              <a href="https://memory.lol/app/">
                <strong>memory.lol</strong>
              </a>{" "}
              are developed by{" "}
              <a href="https://twitter.com/travisbrown">Travis Brown</a>. The
              source code is licensed under the
              <a href="https://anticapitalist.software/">
                {" "}
                Anti-Capitalist Software License
              </a>
              .
            </p>
          </Content>
        </Container>
      </Footer>
    </BrowserRouter>
  );
}

export default App;
