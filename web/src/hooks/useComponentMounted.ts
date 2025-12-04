
// Este hook se encarga de darte la referencia del estado de montaje

import React from "react";

// Nos permite conocer si un componente ha sido desmontado
export const useIsMounted = () => {
  const isMounted = React.useRef(false);

  React.useEffect(() => {
    isMounted.current = true;
    return () => {
      isMounted.current = false
    };
  }, []);

  return isMounted;
};
