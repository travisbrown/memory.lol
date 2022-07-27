import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

export function PostLoginRedirect() {
  let navigate = useNavigate();

  useEffect(() => {
    let redirect_uri = window.sessionStorage.getItem("redirect");

    if (redirect_uri) {
      window.sessionStorage.removeItem("redirect");
      navigate(redirect_uri, { replace: true });
    }
  });

  return <></>;
}
