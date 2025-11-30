import { AxiosInstance } from 'axios';
import { DescriptorUsuario, Horario, RolID, Usuario } from '../modelos/usuarios';
import { UsuarioOutDTO } from '../modelos/dto';
import { Dayjs } from 'dayjs';
import { formatDateTimeForServer } from '../modelos/formatos';

export interface UsuariosApi {
  usuarios(): Promise<Usuario[]>;
  usuario(id: string): Promise<Usuario>;
  actualizar_usuario(usuario: UsuarioOutDTO): Promise<void>;
  crearUsuario(usuario: UsuarioOutDTO): Promise<void>;
  actualizar_password(usuarioId: number, passw: string): Promise<void>;
  login(dni: string, passw: string): Promise<Usuario>;
  logout(id: string): Promise<void>;
  horarioSinAsignar(usuarioId: string, fechaHora: Dayjs): Promise<Horario[]>;
  horarioCercano(usuarioId: string, fechaHora: Dayjs): Promise<Horario[]>;
  usuariosPorRol(id: RolID): Promise<DescriptorUsuario[]>
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
    const response = await this.axios.get('api/usuarios');
    const usuariosData = response.data;

    return Array.isArray(usuariosData)
      ? usuariosData.map(Usuario.fromRequest)
      : [];
  }

  async usuario(id: string): Promise<Usuario> {
    const response = await this.axios.get('api/usuarios/' + id);
    return Usuario.fromRequest(response.data)
  }

  async actualizar_usuario(usuario: UsuarioOutDTO): Promise<void> {
    return this.axios.put('api/usuarios', usuario);
  }

  async crearUsuario(usuario: UsuarioOutDTO): Promise<void> {
    usuario.id = 0; // Los nuevos usuarios asignan el id en el backend
    return this.axios.post('api/usuarios', usuario);
  }

  async actualizar_password(usuarioId: number, passw: string): Promise<void> {
    return this.axios.put(
      'api/usuarios/password',
      {
        id: usuarioId,
        password: passw
      }
    );
  }

  async login(dni: string, passw: string): Promise<Usuario> {
    const response = await this.axios.post(
      'auth/usuarios/login',
      {
        dni: dni,
        password: passw
      }
    );

    if (response.status == 401 || response.status == 500) {
      throw new Error(response.data);
    }

    return Usuario.fromRequest(response.data)
  }

  async logout(id: string): Promise<void> {
    return this.axios.get(`api/usuarios/${id}/logout`);
  }

  async horarioSinAsignar(
    usuarioId: string, fechaHora: Dayjs): Promise<Horario[]> {
    let response;

    response = await this.axios.get(
      `api/usuarios/${usuarioId}/horario/sin/asignar/${formatDateTimeForServer(fechaHora)}`);

    return Array.isArray(response.data)
      ? response.data.map(Horario.fromRequest)
      : [];
  }

  async horarioCercano(
    usuarioId: string, fechaHora: Dayjs): Promise<Horario[]> {
    let response;

    response = await this.axios.get(
      `api/usuarios/${usuarioId}/horario/cercano/${formatDateTimeForServer(fechaHora)}`);

    return Array.isArray(response.data)
      ? response.data.map(Horario.fromRequest)
      : [];
  }

  async usuariosPorRol(id: RolID): Promise<DescriptorUsuario[]> {
    const response = await this.axios.get(`api/roles/${id}/usuarios`);
    const usuariosData = response.data;

    return Array.isArray(usuariosData)
      ? usuariosData.map(DescriptorUsuario.fromRequest)
      : [];
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

  async usuario(id: string): Promise<Usuario> {
    return Usuario.fromRequest({
      id: id,
      dni: '12345678A',
      email: 'davidandsusanaddadaddasda@example.com',
      nombre: 'Juan',
      primer_apellido: 'Pérez',
      segundo_apellido: 'Gómez',
      activo: '2024-01-15',
      inicio: '2024-01-10',
      roles: [1, 2, 3, 4, 5, 6, 7]
    })
  }

  async actualizar_usuario(_: UsuarioOutDTO): Promise<void> {
    return;
  }

  async crearUsuario(_: UsuarioOutDTO): Promise<void> {
    return;
  }

  async actualizar_password(_: number, __: string): Promise<void> {
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

  async horarioSinAsignar(_: string, __: Dayjs): Promise<Horario[]> {
    const horariosFicticios = [
      {
        dia: 'L',
        hora_inicio: '08:00',
        hora_fin: '10:00',
        horas_a_trabajar: 2
      },
      {
        dia: 'L',
        hora_inicio: '12:00',
        hora_fin: '13:00',
        horas_a_trabajar: 1
      },
    ]

    return horariosFicticios.map(Horario.fromRequest);
  }

  async horarioCercano(_: string, __: Dayjs): Promise<Horario[]> {
    const horariosFicticios = [
      {
        dia: 'L',
        hora_inicio: '08:00',
        hora_fin: '10:00',
        horas_a_trabajar: 2
      },
      {
        dia: 'L',
        hora_inicio: '12:00',
        hora_fin: '13:00',
        horas_a_trabajar: 1
      },
    ]

    return horariosFicticios.map(Horario.fromRequest);
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
}