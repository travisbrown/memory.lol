import { ReactElement, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { Container, Table } from "react-bulma-components";

interface AccountResult {
  id: number;
  id_str: string;
  screen_names: { [key: string]: string[] };
}

interface AccountsResult {
  accounts: AccountResult[];
}

function isAccountResult(
  result: AccountResult | AccountsResult | { [key: string]: AccountsResult }
): result is AccountResult {
  return (result as AccountResult).id_str !== undefined;
}

function isAccountsResult(
  result: AccountResult | AccountsResult | { [key: string]: AccountsResult }
): result is AccountsResult {
  return (result as AccountsResult).accounts !== undefined;
}

function renderResults(results: [string | null, AccountResult][]) {
  let blocks = [];

  for (let [query, result] of results) {
    let query_key_part = query ? `-${query}` : "";
    let head_key = `${result.id_str}${query_key_part}-head`;
    let body_key = `${result.id_str}${query_key_part}-body`;
    let title = query ? `${result.id_str} (${query})` : `${result.id_str}`;

    blocks.push(
      <thead key={head_key}>
        <tr>
          <th colSpan={3}>{title}</th>
        </tr>
      </thead>
    );

    let rows = [];

    for (let [key, value] of new Map(Object.entries(result.screen_names))) {
      let first = value && value.length > 0 ? value[0] : "unknown";
      let last =
        value && value.length > 1
          ? value[1]
          : value && value.length > 0
          ? value[0]
          : "unknown";

      rows.push(
        <tr key={`${result.id_str}-${key}`}>
          <td>
            <Link to={`/tw/${key}`}>{key}</Link>
          </td>
          <td>{first}</td>
          <td>{last}</td>
        </tr>
      );
    }

    blocks.push(<tbody key={body_key}>{rows}</tbody>);
  }

  return (
    <Container>
      <Table hoverable striped bordered>
        {blocks}
      </Table>
    </Container>
  );
}

function renderResult(
  result: AccountResult | AccountsResult | { [key: string]: AccountsResult }
): ReactElement<any, any> {
  if (isAccountResult(result)) {
    return renderResults([[null, result]]);
  } else if (isAccountsResult(result)) {
    return renderResults(result.accounts.map((result, _) => [null, result]));
  }

  let map = new Map(Object.entries(result));
  let queries: [string | null, AccountResult][] = [];

  for (let [key, value] of map) {
    for (result of value.accounts) {
      queries.push([key, result]);
    }
  }

  return renderResults(queries);
}

export function TwSearch() {
  const [result, setResult] = useState<
    AccountResult | AccountsResult | { [key: string]: AccountsResult } | null
  >(null);
  const [error, setError] = useState<Error | null>(null);

  let params = useParams();
  const userId = params.userId === undefined ? null : params.userId;
  const screenName = params.screenName === undefined ? null : params.screenName;

  useEffect(() => {
    let query = userId !== null ? `/tw/id/${userId}` : `/tw/${screenName}`;

    fetch(query)
      .then((res) => res.json())
      .then(
        (new_result) => {
          setResult(new_result);
        },
        (new_error) => {
          setError(new_error);
        }
      );
  }, [userId, screenName]);

  if (result) {
    return renderResult(result);
  } else {
    if (error) {
      console.log(error);
    }
    return <></>;
  }
}
