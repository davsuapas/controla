import * as React from 'react';
import * as ReactDOM from 'react-dom/client';
import { StyledEngineProvider } from '@mui/material/styles';
import Controla from './controla';
import dayjs from 'dayjs';
import 'dayjs/locale/es';
import { crearAPI } from './api/usuarios';

dayjs.locale('es');

crearAPI();

ReactDOM.createRoot(document.querySelector("#root")!).render(
  <React.StrictMode>
    <StyledEngineProvider injectFirst>
      <Controla />
    </StyledEngineProvider>
  </React.StrictMode>
);