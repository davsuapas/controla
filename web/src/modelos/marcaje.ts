import dayjs, { Dayjs } from "dayjs"
import { DescriptorUsuario, Horario } from "./usuarios"
import { DominiosWithCacheUsuarioDTO } from "./dto";

export class DescriptorMarcaje {
  constructor(
    public id: number,
    public hora_inicio: string | null,
    public hora_fin: string | null,
  ) { }

  static fromRequest(obj: any): DescriptorMarcaje {
    return new DescriptorMarcaje(
      obj.id,
      obj.hora_inicio,
      obj.hora_fin || null,
    );
  }
}

export class Marcaje {

  constructor(
    public id: number,
    public usuario: DescriptorUsuario,
    public usuario_reg: DescriptorUsuario | null,
    public fecha: Dayjs,
    public horaInicio: string,
    public horaFin: string | null,
    public horario: Horario | null,
    public horasTrabajadas: number | null,
  ) { }

  // Crea una instancia desde la solicitudo del servidor
  static fromRequest(dto: DominiosWithCacheUsuarioDTO<any>): Marcaje[] {
    return dto.items.map(item => {
      return new Marcaje(
        item.id,
        dto.usuario(item.usuario),
        dto.usuarioOptional(item.usuario_reg),
        dayjs(item.fecha),
        item.hora_inicio,
        item.hora_fin || null,
        item.horario ? Horario.fromRequest(item.horario) : null,
        item.hora_trabajadas,
      );
    });
  }

  // Devuelve el usuario que registro el marcaje si no es nulo
  // si no el usuario por defecto
  usuarioCreador(usuarioDefault: DescriptorUsuario): DescriptorUsuario {
    // Por rendimiento no se trae el usuario porque normalmente
    // esta filtrado por el mismo y se puede obtener
    return this.usuario_reg ? this.usuario_reg : usuarioDefault;
  }

  horaFinToStr(): string {
    return this.horaFin ? this.horaFin : '--:--';
  }

  horaTrabajadasToStr(): string {
    return this.horasTrabajadas ? this.horasTrabajadas.toFixed(2)
      : '--';
  }

}