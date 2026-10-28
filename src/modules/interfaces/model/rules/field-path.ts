import { FieldScope } from './field-scope';

export class FieldPath {
  private static readonly scopes: ReadonlyMap<string, FieldScope> = new Map(
    Object.values(FieldScope).map((scope) => [scope, scope]),
  );

  private readonly fieldScope: FieldScope | null;
  private readonly partySlot: number | null;
  private readonly fieldName: string;

  public constructor(path: string) {
    const boundary = path.indexOf('.');

    const prefix = boundary === -1 ? path : path.slice(0, boundary);

    const party = /^party([1-9]|1[01])$/.exec(prefix);

    this.partySlot = party === null ? null : Number(party[1]);
    this.fieldScope =
      this.partySlot === null ? (FieldPath.scopes.get(prefix) ?? null) : FieldScope.Party;
    this.fieldName = boundary === -1 ? '' : path.slice(boundary + 1);
  }

  public scope(): FieldScope | null {
    return this.fieldScope;
  }

  public member(): number | null {
    return this.partySlot;
  }

  public field(): string {
    return this.fieldName;
  }
}
