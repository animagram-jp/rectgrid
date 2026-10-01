# Reference values for the point-to-ellipse tests in src/geometry.rs (ELLIPSE_CASES).
#   python3 scripts/ref_ellipse.py gen     # prints the Rust table rows
#   python3 scripts/ref_ellipse.py audit   # re-evaluates the first rows and compares with brute force
# - inputs are converted EXACTLY (Decimal(float)), i.e. the very doubles the Rust test passes in
# - root found by *geometric* bisection in 1600-digit Decimal (uniform even for roots 100s of decades away)
# - distances cross-checked against brute-force parametric minimisation in plain floats where feasible
from decimal import Decimal as D, getcontext
import math, sys
getcontext().prec = 1600

def root_u(e0, e1, y0, y1):
    d = (e0 - e1) * (e0 + e1)
    n0, n1 = e0 * y0, e1 * y1
    lo = max(n1, n0 - d)
    hi = (n0 * n0 + n1 * n1).sqrt()
    F = lambda u: (n0 / (u + d)) ** 2 + (n1 / u) ** 2 - 1
    for _ in range(900):
        mid = (lo * hi).sqrt()
        if F(mid) > 0: lo = mid
        else: hi = mid
    return (lo * hi).sqrt(), d

def exact_first_quadrant(e0, e1, y0, y1):
    e0, e1, y0, y1 = map(D, (e0, e1, y0, y1))    # exact binary values
    if e0 == e1:
        n = (y0 * y0 + y1 * y1).sqrt()
        x0, x1 = (e0, D(0)) if n == 0 else (e0 * y0 / n, e0 * y1 / n)
    elif y1 > 0 and y0 > 0:
        u, d = root_u(e0, e1, y0, y1)
        x0, x1 = e0 * (e0 * y0 / (u + d)), e1 * (e1 * y1 / u)
    elif y1 > 0:
        x0, x1 = D(0), e1
    else:
        d = (e0 - e1) * (e0 + e1)
        if y0 < d / e0:
            x0 = e0 * e0 * y0 / d; x1 = e1 * (1 - (x0 / e0) ** 2).sqrt()
        else:
            x0, x1 = e0, D(0)
    return x0, x1, ((x0 - y0) ** 2 + (x1 - y1) ** 2).sqrt(), (e0, e1, y0, y1)

def signed(rx, ry, dx, dy):
    swap = ry > rx
    e0, e1 = (ry, rx) if swap else (rx, ry)
    y0, y1 = (abs(dy), abs(dx)) if swap else (abs(dx), abs(dy))
    x0, x1, dist, (E0, E1, Y0, Y1) = exact_first_quadrant(e0, e1, y0, y1)
    fx, fy = (x1, x0) if swap else (x0, x1)
    fx = -fx if dx < 0 else fx; fy = -fy if dy < 0 else fy
    inside = (Y0 / E0) ** 2 + (Y1 / E1) ** 2 < 1
    return (-dist if inside else dist), fx, fy

def brute(a, b, x, y):
    f = lambda t: math.hypot(a * math.cos(t) - x, b * math.sin(t) - y)
    n = 200000
    t0 = min((2 * math.pi * i / n for i in range(n)), key=f)
    lo, hi = t0 - 2 * math.pi / n, t0 + 2 * math.pi / n
    for _ in range(100):
        m1, m2 = lo + (hi - lo) / 3, hi - (hi - lo) / 3
        if f(m1) < f(m2): hi = m2
        else: lo = m1
    t = (lo + hi) / 2
    return f(t), (a * math.cos(t), b * math.sin(t))

OLD = [  # (name, rx, ry, dx, dy, old signed, old foot)
 ("4x1 (3,2)", 4, 1, 3, 2, 1.2973054925552014, (2.7090557089436533, 0.7357401530873434)),
 ("4x1 (10,10)", 4, 1, 10, 10, 11.498686021746554, (3.733379239352857, 0.35898114985061225)),
 ("4x1 (1.5,0.5) inside", 4, 1, 1.5, 0.5, -0.42478368617597306, (1.5442009682459554, 0.9224777561568983)),
 ("4x1 (5,0) on major axis", 4, 1, 5, 0, 1.0, (4.0, 0.0)),
 ("4x1 (0,5) on minor axis", 4, 1, 0, 5, 4.0, (0.0, 1.0)),
 ("1x4 (2,3) axes swapped", 1, 4, 2, 3, 1.2973054925552014, (0.7357401530873434, 2.7090557089436533)),
 ("4x1 (-3,-2) third quadrant", 4, 1, -3, -2, 1.2973054925552014, (-2.7090557089436533, -0.7357401530873434)),
 ("5x5 (6,8) circle", 5, 5, 6, 8, 5.0, (3.0, 4.0)),
 ("4x1 center", 4, 1, 0, 0, -1.0, (0.0, 1.0)),
 ("100x0.01 (50,1) eccentric", 100, 0.01, 50, 1, 0.9913397443099252, (49.99994276506063, 0.00866025734230263)),
 ("100x0.01 (120,0.005) eccentric", 100, 0.01, 120, 0.005, 20.00000062499996, (99.99999999999997, 2.4999998750000017e-10)),
 ("4x1 (3,1e-12) near axis", 4, 1, 3, 1e-12, -0.6324555320327272, (3.1999999999996445, 0.6000000000001185)),
 ("4x1 (3.7,1e-6) near axis", 4, 1, 3.7, 1e-6, -0.29552158605376616, (3.946665050074008, 0.16275652493147685)),
 ("4x1 (0.001,5) near minor axis", 4, 1, 0.001, 5, 4.000000025, (0.000799999996, 0.99999998)),
 ("3x2 (1,1) inside", 3, 2, 1, 1, -0.8554637244664959, (1.249987537492453, 1.8181224938702933)),
 ("4x1 (3,1e-15) on the axis side", 4, 1, 3, 1e-15, -0.6324555320336749, (3.1999999999999997, 0.6000000000000001)),
 ("4x1 (3.7,1e-14) on the axis side", 4, 1, 3.7, 1e-14, -0.2955221367906812, (3.9466666666666503, 0.16275407487647386)),
 ("100x0.01 (50,1e-9) eccentric", 100, 0.01, 50, 1e-9, -0.00866025302341063, (50.00000049999995, 0.008660254008976876)),
]
NEW = [
 ("2x1 cusp (1.5,1e-300)", 2, 1, 1.5, 1e-300),
 ("2x1 cusp (1.5,1e-100)", 2, 1, 1.5, 1e-100),
 ("2x1 cusp (1.5,1e-30)", 2, 1, 1.5, 1e-30),
 ("2x1 cusp (1.5,1e-8)", 2, 1, 1.5, 1e-8),
 ("2x1 just below cusp (1.5(1-1e-9),1e-12)", 2, 1, 1.5 * (1 - 1e-9), 1e-12),
 ("2x1 just above cusp (1.5(1+1e-9),1e-12)", 2, 1, 1.5 * (1 + 1e-9), 1e-12),
 ("2x1 beyond cusp (1.6,1e-6)", 2, 1, 1.6, 1e-6),
 ("near-circle 1+1e-12 inside (700,800)", 1000.000000001, 1000, 700, 800),
 ("near-circle 1+1e-12 outside (3000,4000)", 1000.000000001, 1000, 3000, 4000),
 ("near-circle 1+1e-8 inside (0.6,0.8)", 1.00000001, 1, 0.6, 0.8),
 ("near-circle 1+1e-8 outside (3,4)", 1.00000001, 1, 3, 4),
 ("1e6x1 (5e5,0.5)", 1e6, 1, 5e5, 0.5),
 ("1e6x1 (2e6,3)", 1e6, 1, 2e6, 3),
 ("1e8x1 (3e7,1)", 1e8, 1, 3e7, 1),
 ("4x1 far (1e6,1e6)", 4, 1, 1e6, 1e6),
 ("4x1 far on major side (1e9,1)", 4, 1, 1e9, 1),
 ("4e100x1e100 (3e100,2e100)", 4e100, 1e100, 3e100, 2e100),
 ("4e-100x1e-100 (3e-100,2e-100)", 4e-100, 1e-100, 3e-100, 2e-100),
 ("4x1 (3,1e-320) subnormal y", 4, 1, 3, 1e-320),
 ("4x1 (1e-300,5) tiny x", 4, 1, 1e-300, 5),
 ("4x1 (1e-300,1e-300) tiny both", 4, 1, 1e-300, 1e-300),
]

def fl(x): return float(x)
mode = sys.argv[1] if len(sys.argv) > 1 else "audit"
if mode == "audit":
    print("== old table re-evaluated with EXACT double inputs ==")
    worst = 0
    for name, rx, ry, dx, dy, osd, ofoot in OLD:
        s, fx, fy = signed(rx, ry, dx, dy)
        dd = abs(fl(s) - osd) / (1 + abs(osd)); df = max(abs(fl(fx) - ofoot[0]), abs(fl(fy) - ofoot[1])) / (1 + max(abs(ofoot[0]), abs(ofoot[1])))
        worst = max(worst, dd, df)
        flag = "  <-- differs" if max(dd, df) > 1e-14 else ""
        print(f"{name:36s} dist diff {dd:.1e}  foot diff {df:.1e}{flag}")
    print("worst relative change:", worst)
    print("== foot points vs brute force (independent of the u-root formulation) ==")
    for name, rx, ry, dx, dy, *_ in OLD:
        if abs(dx) < 1e-13 and abs(dy) < 1e-13 or name.startswith("4x1 center"): continue
        s, fx, fy = signed(rx, ry, dx, dy)
        bd, (bx, by) = brute(rx, ry, abs(dx), abs(dy))
        ddiff = abs(bd - abs(fl(s)))
        # foot compare in the |dx|,|dy| frame (brute-force returns the first-quadrant minimiser)
        fdiff = min(math.hypot(abs(fl(fx)) - bx, abs(fl(fy)) - by), math.hypot(abs(fl(fx)) - by, abs(fl(fy)) - bx))
        print(f"{name:36s} dist diff {ddiff:.1e}  foot diff (sqrt-limited) {fdiff:.1e}")
else:
    print("// ---- generated by ref_ellipse2.py (exact-double inputs, 1600-digit geometric bisection) ----")
    for name, rx, ry, dx, dy, *_ in OLD:
        s, fx, fy = signed(rx, ry, dx, dy)
        print(f'        ("{name}", {float(rx)!r}, {float(ry)!r}, {float(dx)!r}, {float(dy)!r}, {fl(s)!r}, ({fl(fx)!r}, {fl(fy)!r})),')
    for name, rx, ry, dx, dy in NEW:
        s, fx, fy = signed(rx, ry, dx, dy)
        print(f'        ("{name}", {float(rx)!r}, {float(ry)!r}, {float(dx)!r}, {float(dy)!r}, {fl(s)!r}, ({fl(fx)!r}, {fl(fy)!r})),')
