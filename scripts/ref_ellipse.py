"""Reference values for the point-to-ellipse tests in src/geometry.rs (ELLIPSE_CASES).

    python3 scripts/ref_ellipse.py gen     prints the Rust table rows
    python3 scripts/ref_ellipse.py audit   compares the reference with a brute-force minimisation

Inputs are converted exactly (Decimal(float)). The root of Eberly's F is found by geometric bisection in
1600-digit Decimal, which stays uniform for roots hundreds of decades away from the bracket end.
"""
import math
import sys
from decimal import Decimal as D, getcontext

getcontext().prec = 1600


def root_u(e0, e1, y0, y1):
    """Root u of F(u) = (e0 y0 / (u + d))^2 + (e1 y1 / u)^2 - 1 with d = e0^2 - e1^2; returns (u, d)."""
    d = (e0 - e1) * (e0 + e1)
    n0, n1 = e0 * y0, e1 * y1
    lo = max(n1, n0 - d)
    hi = (n0 * n0 + n1 * n1).sqrt()
    F = lambda u: (n0 / (u + d)) ** 2 + (n1 / u) ** 2 - 1
    for _ in range(900):
        mid = (lo * hi).sqrt()
        if F(mid) > 0:
            lo = mid
        else:
            hi = mid
    return (lo * hi).sqrt(), d


def exact_first_quadrant(e0, e1, y0, y1):
    """Closest ellipse point and distance for e0 >= e1 > 0 and y0, y1 >= 0; returns (x0, x1, distance, exact inputs)."""
    e0, e1, y0, y1 = map(D, (e0, e1, y0, y1))
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
            x0 = e0 * e0 * y0 / d
            x1 = e1 * (1 - (x0 / e0) ** 2).sqrt()
        else:
            x0, x1 = e0, D(0)
    return x0, x1, ((x0 - y0) ** 2 + (x1 - y1) ** 2).sqrt(), (e0, e1, y0, y1)


def signed(rx, ry, dx, dy):
    """Signed distance (negative inside) and closest point relative to the center, for any quadrant and axis order."""
    swap = ry > rx
    e0, e1 = (ry, rx) if swap else (rx, ry)
    y0, y1 = (abs(dy), abs(dx)) if swap else (abs(dx), abs(dy))
    x0, x1, dist, (E0, E1, Y0, Y1) = exact_first_quadrant(e0, e1, y0, y1)
    fx, fy = (x1, x0) if swap else (x0, x1)
    fx = -fx if dx < 0 else fx
    fy = -fy if dy < 0 else fy
    inside = (Y0 / E0) ** 2 + (Y1 / E1) ** 2 < 1
    return (-dist if inside else dist), fx, fy


def brute(a, b, x, y):
    """Minimum distance from (x, y) to the ellipse by sampling the parameter and refining; returns (distance, point)."""
    f = lambda t: math.hypot(a * math.cos(t) - x, b * math.sin(t) - y)
    n = 200000
    t0 = min((2 * math.pi * i / n for i in range(n)), key=f)
    lo, hi = t0 - 2 * math.pi / n, t0 + 2 * math.pi / n
    for _ in range(100):
        m1, m2 = lo + (hi - lo) / 3, hi - (hi - lo) / 3
        if f(m1) < f(m2):
            hi = m2
        else:
            lo = m1
    t = (lo + hi) / 2
    return f(t), (a * math.cos(t), b * math.sin(t))


CASES = [
 ("4x1 (3,2)", 4, 1, 3, 2),
 ("4x1 (10,10)", 4, 1, 10, 10),
 ("4x1 (1.5,0.5) inside", 4, 1, 1.5, 0.5),
 ("4x1 (5,0) on major axis", 4, 1, 5, 0),
 ("4x1 (0,5) on minor axis", 4, 1, 0, 5),
 ("1x4 (2,3) axes swapped", 1, 4, 2, 3),
 ("4x1 (-3,-2) third quadrant", 4, 1, -3, -2),
 ("5x5 (6,8) circle", 5, 5, 6, 8),
 ("4x1 center", 4, 1, 0, 0),
 ("100x0.01 (50,1) eccentric", 100, 0.01, 50, 1),
 ("100x0.01 (120,0.005) eccentric", 100, 0.01, 120, 0.005),
 ("4x1 (3,1e-12) near axis", 4, 1, 3, 1e-12),
 ("4x1 (3.7,1e-6) near axis", 4, 1, 3.7, 1e-6),
 ("4x1 (0.001,5) near minor axis", 4, 1, 0.001, 5),
 ("3x2 (1,1) inside", 3, 2, 1, 1),
 ("4x1 (3,1e-15) on the axis side", 4, 1, 3, 1e-15),
 ("4x1 (3.7,1e-14) on the axis side", 4, 1, 3.7, 1e-14),
 ("100x0.01 (50,1e-9) eccentric", 100, 0.01, 50, 1e-9),
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


def main(mode):
    for name, rx, ry, dx, dy in CASES:
        s, fx, fy = signed(rx, ry, dx, dy)
        if mode == "gen":
            print(f'        ("{name}", {float(rx)!r}, {float(ry)!r}, {float(dx)!r}, {float(dy)!r}, {float(s)!r}, ({float(fx)!r}, {float(fy)!r})),')
        elif abs(dx) > 1e-13 or abs(dy) > 1e-13:
            bd, _ = brute(rx, ry, abs(dx), abs(dy))
            print(f"{name:42s} |brute - reference| = {abs(bd - abs(float(s))):.1e}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "audit")
