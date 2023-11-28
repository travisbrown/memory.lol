import { Button, Form } from "react-bulma-components";
import { useNavigate } from "react-router-dom";
import { useState } from "react";

export function TwForm() {
  const navigate = useNavigate();
  const [searchFields, setSearchFields] = useState({
    screen_name: "",
    twitter_id: "",
  });

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setSearchFields({
      ...searchFields,
      [event.target.name]: event.target.value,
    });
  };

  const handleScreenNameSearch = (
    event: React.MouseEvent<HTMLButtonElement>
  ) => {
    event.preventDefault();
    if (searchFields.screen_name !== "") {
      navigate(`tw/${searchFields.screen_name}`);
    }
  };

  const handleTwitterIdSearch = (
    event: React.MouseEvent<HTMLButtonElement>
  ) => {
    event.preventDefault();
    if (searchFields.twitter_id !== "") {
      navigate(`tw/id/${searchFields.twitter_id}`);
    }
  };

  return (
    <form>
      <Form.Field kind="addons">
        <Form.Control>
          <Form.Input
            placeholder="Screen name"
            type="search"
            name="screen_name"
            value={searchFields.screen_name}
            onChange={handleChange}
          />
        </Form.Control>
        <Form.Control>
          <Button color="info" onClick={handleScreenNameSearch}>
            Search
          </Button>
        </Form.Control>
      </Form.Field>
      <Form.Field kind="addons">
        <Form.Control>
          <Form.Input
            placeholder="Twitter ID"
            type="search"
            name="twitter_id"
            value={searchFields.twitter_id}
            onChange={handleChange}
          />
        </Form.Control>
        <Form.Control>
          <Button color="info" onClick={handleTwitterIdSearch}>
            Search
          </Button>
        </Form.Control>
      </Form.Field>
    </form>
  );
}
