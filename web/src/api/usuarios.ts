import { AxiosInstance } from 'axios';
import { ConfigHorario, DescriptorUsuario, DiaSemana, Horario, RolID, Usuario } from '../modelos/usuarios';
import { UsuarioOutDTO } from '../modelos/dto';
import dayjs, { Dayjs } from 'dayjs';
import { formatDateForServer, formatDateTimeForServer } from '../modelos/formatos';
import { ConfigRequest } from '../net/interceptor';

export interface UsuariosApi {
  login(dni: string, passw: string): Promise<Usuario>;
  logout(id: string): Promise<void>;
  usuarios(): Promise<Usuario[]>;
  // todosLosCalendarios si es true se envían todos y se marcan
  // los asignados a el usuario, si es false solo los asignados
  usuario(id: string, todosLosCalendarios: boolean): Promise<Usuario>;
  crearUsuario(usuario: UsuarioOutDTO): Promise<void>;
  actualizarUsuario(usuario: UsuarioOutDTO): Promise<void>;
  actualizarPassword(usuarioId: number, passw: string): Promise<void>;
  horarioCercano(usuarioId: string, fechaHora: Dayjs): Promise<Horario | null>;
  usuariosPorRol(id: RolID): Promise<DescriptorUsuario[]>;
  duplicarHorario(usuarioId: number, fechaCreacion: Dayjs): Promise<ConfigHorario[]>;
  horarios(usuarioId: number): Promise<ConfigHorario[]>;
  horario(id: number): Promise<ConfigHorario>;
  crearHorario(horario: ConfigHorario): Promise<void>;
  actualizarHorario(horario: ConfigHorario): Promise<void>;
  eliminarHorario(id: number): Promise<void>;
}

export class ContextoApi {
  constructor(public usuarios: UsuariosApi) {
  }
}

// Implementación de UsuariosApi en modo producción
export class UsuariosAxiosApi implements UsuariosApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async usuarios(): Promise<Usuario[]> {
    const response = await this.axios.get('usuarios');
    const usuariosData = response.data;

    return Array.isArray(usuariosData)
      ? usuariosData.map(Usuario.fromRequest)
      : [];
  }

  async usuario(id: string, todosLosCalendarios: boolean): Promise<Usuario> {
    const response = await this.axios.get('usuarios/' + id, {
      params: { todos_los_calendarios: todosLosCalendarios }
    });
    return Usuario.fromRequest(response.data)
  }

  async crearUsuario(usuario: UsuarioOutDTO): Promise<void> {
    usuario.id = 0; // Los nuevos usuarios asignan el id en el backend
    return this.axios.post('usuarios', usuario);
  }

  async actualizarUsuario(usuario: UsuarioOutDTO): Promise<void> {
    return this.axios.put('usuarios', usuario);
  }

  async actualizarPassword(usuarioId: number, passw: string): Promise<void> {
    return this.axios.put(
      'usuarios/password',
      {
        id: usuarioId,
        password: passw
      }
    );
  }

  async login(dni: string, passw: string): Promise<Usuario> {
    // Modificamos la baseURL para quitar '/api' y apuntar a '/auth'
    const baseURL = this.axios.defaults.baseURL?.replace(/\/api$/, '');

    const response = await this.axios.post(
      'auth/usuarios/login',
      {
        dni: dni,
        password: passw
      },
      { baseURL }
    );

    if (response.status == 401 || response.status == 500) {
      throw new Error(response.data);
    }

    return Usuario.fromRequest(response.data)
  }

  async logout(id: string): Promise<void> {
    return this.axios.get(`usuarios/${id}/logout`);
  }

  async horarioCercano(
    usuarioId: string, fechaHora: Dayjs): Promise<Horario | null> {
    try {
      const response = await this.axios.get(
        `usuarios/${usuarioId}/horario/cercano/${formatDateTimeForServer(fechaHora)}`,
        { manejarErrorInesperado: true } as ConfigRequest,
      );
      return Horario.fromRequest(response.data);
    } catch (error: any) {
      if (error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  }

  async usuariosPorRol(id: RolID): Promise<DescriptorUsuario[]> {
    const response = await this.axios.get(`roles/${id}/usuarios`);
    const usuariosData = response.data;

    return Array.isArray(usuariosData)
      ? usuariosData.map(DescriptorUsuario.fromRequest)
      : [];
  }

  async duplicarHorario(usuarioId: number, fechaCreacion: Dayjs): Promise<ConfigHorario[]> {
    const response = await this.axios.post(
      `usuarios/${usuarioId}/horarios/duplicar/${formatDateForServer(fechaCreacion)}`
    );

    return Array.isArray(response.data)
      ? ConfigHorario.fromRequestArray(response.data) : [];
  }

  async horarios(usuarioId: number): Promise<ConfigHorario[]> {
    const response = await this.axios.get(`usuarios/${usuarioId}/horarios`);

    return Array.isArray(response.data)
      ? ConfigHorario.fromRequestArray(response.data)
      : [];
  }

  async horario(id: number): Promise<ConfigHorario> {
    const response = await this.axios.get(`horarios/${id}`);
    return ConfigHorario.fromRequest(response.data);
  }

  async crearHorario(horario: ConfigHorario): Promise<void> {
    return this.axios.post('horarios', horario.toServer());
  }

  async actualizarHorario(horario: ConfigHorario): Promise<void> {
    return this.axios.put('horarios', horario.toServer());
  }

  async eliminarHorario(id: number): Promise<void> {
    return this.axios.delete(`horarios/${id}`);
  }
}

// Implementación de UsuariosApi en modo test
export class UsuariosTestApi implements UsuariosApi {
  async usuarios(): Promise<Usuario[]> {
    // Simular un pequeño retraso de red
    await new Promise(resolve => setTimeout(resolve, 100));

    const usuariosFicticios = [
      {
        id: 1,
        dni: '12345678A',
        email: 'M0q6T@example.com',
        nombre: 'Juan',
        primer_apellido: 'Pérez',
        segundo_apellido: 'Gómez',
        activo: '2024-01-15',
        inicio: '2024-01-10',
        roles: [1, 2, 3, 4, 5, 6, 7]
      },
      {
        id: 2,
        dni: '87654321B',
        email: 'M0q6T@example.com',
        nombre: 'María',
        primer_apellido: 'López',
        segundo_apellido: 'Martínez',
        activo: '2024-02-20',
        inicio: '2024-02-15',
        roles: [1, 2, 3, 4]
      },
      {
        id: 3,
        dni: '11223344C',
        email: 'M0q6T@example.com',
        nombre: 'Carlos',
        primer_apellido: 'García',
        segundo_apellido: null,
        activo: null,
        inicio: '2024-03-05',
        roles: [4, 5, 6]
      },
      {
        id: 4,
        dni: '44332211D',
        email: 'M0q6T@example.com',
        nombre: 'Ana',
        primer_apellido: 'Rodríguez',
        segundo_apellido: 'Fernández',
        activo: '2024-01-30',
        inicio: null,
        roles: [6, 7, 1, 4, 5]
      }
    ];

    return usuariosFicticios.map(Usuario.fromRequest);
  }

  usuario(id: string, _: boolean): Promise<Usuario> {
    return Promise.resolve(Usuario.fromRequest({
      id: id,
      dni: '12345678A',
      email: 'davidandsusanaddadaddasda@example.com',
      nombre: 'Juan',
      primer_apellido: 'Pérez',
      segundo_apellido: 'Gómez',
      activo: '2024-01-15',
      inicio: '2024-01-10',
      roles: [1, 2, 3, 4, 5, 6, 7],
      calendarios: [
        { calendario: 1, nombre: 'Calendario 1', asignado: true },
        { calendario: 2, nombre: 'Calendario 2', asignado: false },
        { calendario: 3, nombre: 'Calendario 3', asignado: true },
      ]
    }));
  }

  async actualizarUsuario(_: UsuarioOutDTO): Promise<void> {
    return;
  }

  async crearUsuario(_: UsuarioOutDTO): Promise<void> {
    return;
  }

  async actualizarPassword(_: number, __: string): Promise<void> {
    return;
  }

  async login(_: string, __: string): Promise<Usuario> {
    return Usuario.fromRequest({
      id: 1,
      dni: '12345678A',
      email: 'M0q6T@example.com',
      nombre: 'Juan',
      primer_apellido: 'Pérez',
      segundo_apellido: 'Gómez',
      activo: '2024-01-15',
      inicio: '2024-01-10',
      roles: [1, 2, 3, 4, 5, 6, 7]
    })
  }

  async logout(_: string): Promise<void> {
    return;
  }

  async horarioCercano(_: string, __: Dayjs): Promise<Horario | null> {
    return Horario.fromRequest(
      {
        dia: 'L',
        horas: 2
      });
  }

  async usuariosPorRol(_: RolID): Promise<DescriptorUsuario[]> {
    const usuariosFicticios = [
      {
        id: 1,
        nombre: 'Juan',
        primer_apellido: 'Pérez',
        segundo_apellido: 'Gómez',
      },
      {
        id: 2,
        nombre: 'María',
        primer_apellido: 'López',
        segundo_apellido: 'Martínez',
      },
    ]

    return usuariosFicticios.map(DescriptorUsuario.fromRequest);
  }

  async duplicarHorario(usuarioId: number, fechaCreacion: Dayjs): Promise<ConfigHorario[]> {
    const lista: ConfigHorario[] = [];
    for (let i = 1; i <= 5; i++) {
      lista.push(new ConfigHorario({
        id: i,
        usuario: usuarioId,
        horario: new Horario({
          dia: DiaSemana.Lunes,
          horas: 2,
        }),
        cortesia: 0,
        fechaCreacion: fechaCreacion,
        caducidadFechaIni: null,
        caducidadFechaFin: null
      }));
    }

    lista.push(new ConfigHorario({
      id: 6,
      usuario: usuarioId,
      horario: new Horario({
        dia: DiaSemana.Martes,
        horas: 5,
      }),
      fechaCreacion: fechaCreacion,
      cortesia: 3,
      caducidadFechaIni: dayjs().add(7, 'days'),
      caducidadFechaFin: dayjs().add(7, 'days')
    }));
    return lista;
  }

  async horarios(usuarioId: number): Promise<ConfigHorario[]> {
    const fecha = dayjs();
    return [
      new ConfigHorario({
        id: 101,
        usuario: usuarioId,
        horario: new Horario({
          dia: DiaSemana.Lunes,
          horas: 3,
        }),
        cortesia: 0,
        fechaCreacion: fecha,
        caducidadFechaIni: dayjs().add(7, 'days'),
        caducidadFechaFin: fecha.add(1, 'year')
      }),
      new ConfigHorario({
        id: 102,
        usuario: usuarioId,
        horario: new Horario({
          dia: DiaSemana.Martes,
          horas: 5,
        }),
        fechaCreacion: fecha,
        caducidadFechaIni: dayjs().add(7, 'days'),
        caducidadFechaFin: fecha.add(6, 'month')
      })
    ];
  }

  async horario(id: number): Promise<ConfigHorario> {
    const fecha = dayjs();
    return new ConfigHorario({
      id: id,
      usuario: 1,
      horario: new Horario({
        dia: DiaSemana.Jueves,
        horas: 2,
      }),
      cortesia: 7,
      fechaCreacion: fecha,
      caducidadFechaIni: dayjs().add(7, 'days'),
      caducidadFechaFin: fecha.add(1, 'year')
    });
  }

  async actualizarHorario(_: ConfigHorario): Promise<void> {
    return;
  }

  async crearHorario(_: ConfigHorario): Promise<void> {
    return;
  }

  async eliminarHorario(_: number): Promise<void> {
    return;
  }
}