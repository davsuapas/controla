import axios from "axios";
import { UsuariosApi, UsuariosAxiosApi, UsuariosTestApi } from "./usuarios";
import { MarcajeApi, MarcajeAxiosApi, MarcajeTestApi } from "./marcaje";
import { IncidenciaApi, IncidenciaAxiosApi, IncidenciaTestApi } from "./incidencias";
import { CalendariosApi, CalendariosAxiosApi, CalendariosTestApi } from "./calendario";
import { InformeApi, InformeAxiosApi, InformeTestApi } from "./informe";

export class ContextoApi {
  constructor(
    public usuarios: UsuariosApi,
    public marcajes: MarcajeApi,
    public inc: IncidenciaApi,
    public calendar: CalendariosApi,
    public informe: InformeApi) {
  }
}

// Variable global para el api (singleton)
let _api: ContextoApi | null = null;

export function api(): ContextoApi {
  if (_api == null) {
    throw Error('No se ha inicializado el API');
  }

  return _api;
}

// Crea el API de acceso a los servicios y lo inicializa
export function crearAPI(modoTest: boolean = false) {
  let usuarioApi: UsuariosApi;
  let marcajeApi: MarcajeApi;
  let incApi: IncidenciaApi;
  let calendarApi: CalendariosApi;
  let informeApi: InformeApi;

  if (modoTest) {
    usuarioApi = new UsuariosTestApi();
    marcajeApi = new MarcajeTestApi();
    incApi = new IncidenciaTestApi();
    calendarApi = new CalendariosTestApi();
    informeApi = new InformeTestApi();
  } else {
    usuarioApi = new UsuariosAxiosApi(axios);
    marcajeApi = new MarcajeAxiosApi(axios);
    incApi = new IncidenciaAxiosApi(axios);
    calendarApi = new CalendariosAxiosApi(axios);
    informeApi = new InformeAxiosApi(axios);
  }

  const contexto = new ContextoApi(
    usuarioApi, marcajeApi, incApi, calendarApi, informeApi);
  _api = contexto;
}
