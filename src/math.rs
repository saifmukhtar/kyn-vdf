//! Imaginary Quadratic Class Group arithmetic with Shanks' NUCOMP and NUDUPL algorithms.
//!
//! This module implements binary quadratic forms $(a, b, c)$ over a negative fundamental
//! discriminant $D = b^2 - 4ac < 0$. The group law is given by Shanks' sub-quadratic
//! composition algorithms NUCOMP and NUDUPL, which use a partial Euclidean reduction
//! step controlled by a threshold $L = \lfloor |D|^{1/4} \rfloor$ to bound intermediate
//! coefficient sizes.
//!
//! ## References
//! - Daniel Shanks (1989): *On Gauss and Composition I, II*
//! - Henri Cohen (1993): *A Course in Computational Algebraic Number Theory*, GTM 138, §5.4
//! - Lipa Long: chiavdf C++ reference implementation

use num_bigint::BigInt;
use num_traits::{Zero, One, Signed};

/// Computes $\lfloor \sqrt{\sqrt{n}} \rfloor = \lfloor |D|^{1/4} \rfloor$.
///
/// Used as the Shanks NUCOMP/NUDUPL threshold $L$ to bound intermediate form
/// coefficients during partial Euclidean reduction steps.
///
/// # Panics
/// Does not panic. Returns `BigInt::zero()` for `n = 0`.
#[inline]
pub fn isqrt_fourth(n: &BigInt) -> BigInt {
    let s1 = n.sqrt();
    s1.sqrt()
}

/// Partial Extended Euclidean Algorithm (partial XGCD).
///
/// Runs the standard Extended GCD algorithm on `(r2, r1)` but **stops early**
/// once the remainder `r1` drops at or below the threshold `l`.
///
/// This is the core primitive used by both NUCOMP and NUDUPL to achieve
/// sub-quadratic composition via partial Euclidean reduction.
///
/// # Parameters
/// - `r2`: The larger initial value (typically the form's `a` coefficient).
/// - `r1`: The smaller initial value (typically the intermediate `k`).
/// - `l`: The stopping threshold $L = \lfloor |D|^{1/4} \rfloor$.
///
/// # Returns
/// A tuple `(co2, co1, r2, r1)` where:
/// - `co2`, `co1` are the Bézout coefficients at the stopping point:
///   `co2·(original r2) + co1·(original r1) = r2` (at stop).
/// - `r2` is the last remainder *before* the threshold was crossed.
/// - `r1` is the last remainder *at or below* the threshold (may be zero if
///   the GCD terminated early).
pub fn xgcd_partial(r2: &BigInt, r1: &BigInt, l: &BigInt) -> (BigInt, BigInt, BigInt, BigInt) {
    let mut r2_cur = r2.clone();
    let mut r1_cur = r1.clone();
    let mut co2 = BigInt::zero();
    let mut co1 = BigInt::from(-1);

    while !r1_cur.is_zero() && &r1_cur > l {
        let q = &r2_cur / &r1_cur;
        let t1 = &r2_cur - &q * &r1_cur;
        let t2 = &co2 - &q * &co1;
        r2_cur = r1_cur;
        r1_cur = t1;
        co2 = co1;
        co1 = t2;
    }
    (co2, co1, r2_cur, r1_cur)
}

/// A binary quadratic form $f = ax^2 + bxy + cy^2$ over an imaginary quadratic field.
///
/// Forms are elements of the class group $\text{Cl}(D)$ of a negative fundamental
/// discriminant $D = b^2 - 4ac < 0$. The group law is composition, and the identity
/// element is the principal form $(1, 1, (1-D)/4)$.
///
/// All arithmetic operations (composition, squaring, exponentiation) produce
/// **reduced** forms, meaning:
/// - $-a < b \le a$
/// - $a \le c$
/// - if $a = c$ then $b \ge 0$
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Form {
    /// The leading coefficient $a > 0$.
    pub a: BigInt,
    /// The cross-term coefficient $b$ (satisfies $-a < b \le a$ when reduced).
    pub b: BigInt,
    /// The constant coefficient $c > 0$ (satisfies $c \ge a$ when reduced).
    pub c: BigInt,
}

impl Form {
    /// Creates a new form $(a, b, c)$ without any validation or reduction.
    ///
    /// Prefer [`Form::from_abd`] or [`Form::generator`] when constructing from
    /// external data, as they validate the discriminant identity.
    pub fn new(a: BigInt, b: BigInt, c: BigInt) -> Self {
        Self { a, b, c }
    }

    /// Returns the principal identity element $(1, 1, (1 - D) / 4)$ for discriminant $D$.
    ///
    /// The identity satisfies $e \circ f = f$ for any form $f$ in the same class group.
    /// Requires $D \equiv 3 \pmod 4$ (standard for imaginary quadratic fields with
    /// fundamental discriminant).
    pub fn identity(d: &BigInt) -> Self {
        let one = BigInt::one();
        let four = BigInt::from(4);
        let c = (&one - d) / four;
        Self {
            a: one.clone(),
            b: one,
            c,
        }
    }

    /// Constructs a form $(a, b, c)$ from $a$, $b$, and discriminant $D$,
    /// computing $c = (b^2 - D) / (4a)$ and verifying exact divisibility.
    ///
    /// Returns `None` if:
    /// - `a` is zero (degenerate form)
    /// - $(b^2 - D)$ is not exactly divisible by $4a$
    pub fn from_abd(a: &BigInt, b: &BigInt, d: &BigInt) -> Option<Self> {
        if a.is_zero() {
            return None;
        }
        let num = b * b - d;
        let den = a * BigInt::from(4);
        if &num % &den != BigInt::zero() {
            return None;
        }
        let c = num / den;
        Some(Self {
            a: a.clone(),
            b: b.clone(),
            c,
        })
    }

    /// Returns the canonical generator form $(2, 1, (1 - D) / 8)$.
    ///
    /// Valid for prime discriminants $D = -p$ where $p \equiv 7 \pmod 8$.
    /// In this case $2$ splits in $\mathbb{Q}(\sqrt{D})$ and the form $(2, 1, \cdot)$
    /// generates a subgroup of the class group of order $\text{ord}(2)$ in $\text{Cl}(D)$.
    ///
    /// Returns `None` if $(1 - D)$ is not divisible by 8, i.e. $D$ is not a valid
    /// prime discriminant of the required form.
    pub fn generator(d: &BigInt) -> Option<Self> {
        let num = BigInt::one() - d;
        let eight = BigInt::from(8);
        if &num % &eight != BigInt::zero() {
            return None;
        }
        let c = num / eight;
        Some(Self {
            a: BigInt::from(2),
            b: BigInt::one(),
            c,
        })
    }

    /// Returns `true` if the form is in reduced normal form.
    ///
    /// A form $(a, b, c)$ is **reduced** if and only if:
    /// - $a > 0$ and $c > 0$
    /// - $-a < b \le a$ (normalization condition)
    /// - $a \le c$ (minimality condition)
    /// - if $a = c$, then $b \ge 0$ (uniqueness condition)
    pub fn is_reduced(&self) -> bool {
        if self.a <= BigInt::zero() || self.c <= BigInt::zero() {
            return false;
        }
        if self.b <= -(&self.a) || self.b > self.a {
            return false;
        }
        if self.a > self.c {
            return false;
        }
        if self.a == self.c && self.b < BigInt::zero() {
            return false;
        }
        true
    }

    /// Reduces this form in place using the Euclidean-style Gauss reduction algorithm.
    ///
    /// After reduction the form satisfies the standard reduced form conditions
    /// (see [`Form::is_reduced`]). Every form class has a unique reduced representative.
    ///
    /// # Algorithm
    /// Iterates two steps until the form is reduced:
    /// 1. **Normalize** $b$: compute $s = \lfloor (a - b) / 2a \rfloor$, then
    ///    $b' = b + 2as$, $c' = (b'^2 - D) / 4a$.
    /// 2. **Minimize** $a$: if $a > c'$, swap and negate — set $(a, b, c) = (c', -b', a)$ — and loop.
    ///
    /// Terminates because the $a$-coefficient strictly decreases each swap.
    pub fn reduce(&mut self, d: &BigInt) {
        use num_integer::Integer;
        let two = BigInt::from(2);
        let four = BigInt::from(4);

        loop {
            // Step 1: normalize b into (-a, a]
            let a2 = &self.a * &two;
            let s = (&self.a - &self.b).div_floor(&a2);
            let b_new = &self.b + &a2 * &s;
            let c_new = (&b_new * &b_new - d) / (&self.a * &four);
            let a_new = self.a.clone();

            // Step 2: if a > c after normalization, swap (a, c) and negate b, then loop
            if a_new > c_new {
                self.a = c_new;
                self.c = a_new;
                self.b = -b_new;
                continue;
            }

            // Step 3: uniqueness — if a == c ensure b ≥ 0
            if a_new == c_new && b_new.is_negative() {
                self.b = -b_new;
            } else {
                self.b = b_new;
            }
            self.a = a_new;
            self.c = c_new;
            break;
        }
    }

    /// Shanks' **NUDUPL** algorithm — fast squaring of a binary quadratic form.
    ///
    /// Computes $f^2 = f \circ f$ in the class group without full reduction of
    /// intermediate results, using a partial Euclidean step controlled by threshold `l`.
    ///
    /// # Algorithm (Shanks, 1989 — Cohen §5.4.2)
    /// Given form $(a_1, b_1, c_1)$ and threshold $L$:
    /// 1. Compute $s = \gcd(b_1, a_1)$ via extended GCD and co-factor $k = -xc_1$.
    /// 2. If $s > 1$: divide $a_1$ by $s$, multiply $c_1$ by $s$.
    /// 3. Reduce $k \pmod{a_1}$.
    /// 4. If $a_1 < L$: use direct multiplication (no partial reduction).
    /// 5. Otherwise: run partial XGCD on $(a_1, k)$ stopping at $L$, then compute the
    ///    new $(a, b, c)$ from the Bézout coefficients `(co2, co1)` and remainders `(r2, r1)`.
    ///
    /// # Parameters
    /// - `d`: The negative fundamental discriminant.
    /// - `l`: The Shanks threshold $L = \lfloor |D|^{1/4} \rfloor$.
    ///
    /// # Returns
    /// A (possibly unreduced) form. Callers must call [`Form::reduce`] to obtain the
    /// canonical representative. Use [`Form::square`] for the combined squaring+reduction.
    pub fn nudupl(&self, d: &BigInt, l: &BigInt) -> Form {
        use num_integer::Integer;
        let two = BigInt::from(2);
        let four = BigInt::from(4);

        let mut a1 = self.a.clone();
        let mut c1 = self.c.clone();

        // Extended GCD of |b| and a to find the co-factor s = gcd(b, a)
        // and Bézout coefficient x such that x·|b| + y·a = s
        let ext = if self.b.is_negative() {
            let b_abs = -(&self.b);
            let e = b_abs.extended_gcd(&a1);
            (-e.x, e.gcd)
        } else {
            let e = self.b.extended_gcd(&a1);
            (e.x, e.gcd)
        };

        // k = -x·c1 (initial value before partial reduction)
        let mut k = -(&ext.0 * &c1);
        let s = ext.1; // s = gcd(b, a)

        // If s > 1, divide out the common factor
        if s != BigInt::one() {
            a1 /= &s;
            c1 *= &s;
        }

        // Normalize k into [0, a1)
        k = k.mod_floor(&a1);

        if a1 < *l {
            // Small a1: direct composition (no partial XGCD needed)
            let t = &a1 * &k;
            let res_a = &a1 * &a1;
            let cb = &two * &t + &self.b;
            let res_c = ((&self.b + &t) * &k + &c1) / &a1;
            Form::new(res_a, cb, res_c)
        } else {
            // Large a1: partial XGCD stops when remainder ≤ L
            // Returns Bézout coefficients (co2, co1) and remainders (r2, r1) at stop
            let (co2, co1, _r2, r1) = xgcd_partial(&a1, &k, l);

            // Compute auxiliary value m2 = (b·r1 - c1·co1) / a1
            let m2 = (&self.b * &r1 - &c1 * &co1) / &a1;

            // New a coefficient: r1² - co1·m2 (sign adjusted to ensure positivity)
            let mut res_a = &r1 * &r1 - &co1 * &m2;
            if !co1.is_negative() {
                res_a = -res_a;
            }

            // Recover b from the partial XGCD Bézout relation
            let cb_num = &two * (&a1 * &r1 - &res_a * &co2);
            let cb = (cb_num / &co1 - &self.b).mod_floor(&(&res_a * &two));

            // Compute c from the discriminant identity: c = (b² - D) / 4a
            let mut res_c = (&cb * &cb - d) / (&res_a * &four);

            // Ensure a > 0 (normalize sign)
            if res_a.is_negative() {
                res_a = -res_a;
                res_c = -res_c;
            }

            Form::new(res_a, cb, res_c)
        }
    }

    /// Shanks' **NUCOMP** algorithm — fast composition of two binary quadratic forms.
    ///
    /// Computes $f_1 \circ f_2$ in the class group using a partial Euclidean reduction
    /// step, keeping intermediate coefficients bounded by threshold `l`.
    ///
    /// # Algorithm (Shanks, 1989 — Cohen §5.4.1)
    /// Given forms $f_1 = (a_1, b_1, c_1)$ and $f_2 = (a_2, b_2, c_2)$ with $a_1 \le a_2$:
    /// 1. Compute $ss = (b_1 + b_2)/2$, $m = (b_1 - b_2)/2$.
    /// 2. Compute $sp = \gcd(a_2 \bmod a_1, a_1)$ via extended GCD.
    /// 3. If $sp = 1$: set $k = m \cdot v_1 \bmod a_1$.
    /// 4. If $sp > 1$: reduce through a second extended GCD of $ss$ and $sp$,
    ///    divide $a_1, a_2$ by $s = \gcd(ss, sp)$ and scale $c_2$.
    /// 5. Apply partial XGCD or direct multiplication depending on $a_1$ vs $L$.
    ///
    /// # Parameters
    /// - `other`: The second form $f_2$ to compose with.
    /// - `d`: The negative fundamental discriminant.
    /// - `l`: The Shanks threshold $L = \lfloor |D|^{1/4} \rfloor$.
    ///
    /// # Returns
    /// A (possibly unreduced) form. Use [`Form::compose`] for the combined compose+reduction.
    pub fn nucomp(&self, other: &Form, d: &BigInt, l: &BigInt) -> Form {
        use num_integer::Integer;
        // Enforce a1 ≤ a2 for the algorithm's precondition
        if self.a > other.a {
            return other.nucomp(self, d, l);
        }

        let two = BigInt::from(2);
        let four = BigInt::from(4);

        let mut a1 = self.a.clone();
        let mut a2 = other.a.clone();
        let mut c2 = other.c.clone();

        // ss = (b1 + b2) / 2,  m = (b1 - b2) / 2
        let ss = (&self.b + &other.b) / &two;
        let m = (&self.b - &other.b) / &two;

        // Compute sp = gcd(a2 mod a1, a1) and Bézout coefficient v1
        let t = a2.mod_floor(&a1);
        let (v1, sp) = if t.is_zero() {
            (BigInt::zero(), a1.clone())
        } else {
            let e = t.extended_gcd(&a1);
            (e.x, e.gcd)
        };

        // Initial k = m·v1 mod a1
        let mut k = (&m * &v1).mod_floor(&a1);

        if sp != BigInt::one() {
            // sp > 1: second GCD step to remove common factor from (ss, sp)
            let e2 = ss.extended_gcd(&sp);
            let v2 = e2.x; // Bézout: v2·ss + u2·sp = s
            let u2 = e2.y;
            let s = e2.gcd;
            // Merge: k = k·u2 - v2·c2
            k = &k * &u2 - &v2 * &c2;
            if s != BigInt::one() {
                // Divide out common factor s from both a's and scale c2
                a1 /= &s;
                a2 /= &s;
                c2 *= &s;
            }
            k = k.mod_floor(&a1);
        }

        if a1 < *l {
            // Small a1: direct multiplication (no partial XGCD needed)
            let t_val = &a2 * &k;
            let ca = &a2 * &a1;
            let cb = &two * &t_val + &other.b;
            let cc = ((&other.b + &t_val) * &k + &c2) / &a1;
            Form::new(ca, cb, cc)
        } else {
            // Large a1: partial XGCD on (a1, k) stopping at L
            let (co2, co1, _r2, r1) = xgcd_partial(&a1, &k, l);

            // Auxiliary values from Bézout coefficients
            let m1 = (&m * &co1 + &a2 * &r1) / &a1;
            let m2 = (&ss * &r1 - &c2 * &co1) / &a1;

            // New a coefficient: r1·m1 - co1·m2 (sign adjusted for positivity)
            let mut ca = &r1 * &m1 - &co1 * &m2;
            if !co1.is_negative() {
                ca = -ca;
            }

            // Recover b from the Bézout relation
            let t_val = &a2 * &r1;
            let cb_num = &two * (&t_val - &ca * &co2);
            let cb = (cb_num / &co1 - &other.b).mod_floor(&(&ca * &two));

            // Compute c from the discriminant identity: c = (b² - D) / 4a
            let mut cc = (&cb * &cb - d) / (&ca * &four);

            // Ensure a > 0
            if ca.is_negative() {
                ca = -ca;
                cc = -cc;
            }

            Form::new(ca, cb, cc)
        }
    }

    /// Fast binary exponentiation using NUDUPL and NUCOMP with threshold-based reduction.
    ///
    /// Computes $f^n$ using the standard left-to-right binary method:
    /// - Start with $\text{result} = f$ (from the leading bit of $n$).
    /// - For each remaining bit $i$ from high to low:
    ///   - Square: $\text{result} \leftarrow \text{NUDUPL}(\text{result})$
    ///   - If bit $i$ is set: compose $\text{result} \leftarrow \text{NUCOMP}(\text{result}, f)$
    ///   - Opportunistically reduce when `result.a.bits() > |D|.bits() / 2`
    ///     to prevent coefficient blowup between full reductions.
    /// - Final full Gauss reduction before return.
    ///
    /// Returns the identity form for `exp = 0`.
    ///
    /// # Parameters
    /// - `exp`: Non-negative exponent as `BigUint`.
    /// - `d`: The negative fundamental discriminant.
    /// - `l`: The Shanks threshold $L = \lfloor |D|^{1/4} \rfloor$.
    pub fn fast_pow(&self, exp: &num_bigint::BigUint, d: &BigInt, l: &BigInt) -> Form {
        use num_traits::Zero;
        if exp.is_zero() {
            return Self::identity(d);
        }

        let mut res = self.clone();
        // Opportunistic reduction threshold: reduce when a exceeds half the discriminant bit-width
        let max_bits = d.abs().bits() / 2;
        let num_bits = exp.bits();

        if num_bits > 1 {
            for i in (0..num_bits - 1).rev() {
                res = res.nudupl(d, l);
                // Bound growth: reduce if a coefficient is getting too large
                if res.a.bits() > max_bits {
                    res.reduce(d);
                }

                if exp.bit(i) {
                    res = res.nucomp(self, d, l);
                }
            }
        }

        // Final canonical reduction
        res.reduce(d);
        res
    }

    /// Raises this form to the power of `exp` using [`Form::fast_pow`].
    ///
    /// Computes the Shanks threshold $L = \lfloor |D|^{1/4} \rfloor$ internally.
    /// Returns the identity form when `exp = 0`.
    pub fn pow(&self, exp: &num_bigint::BigUint, d: &BigInt) -> Self {
        let l = isqrt_fourth(&d.abs());
        self.fast_pow(exp, d, &l)
    }

    /// Composes this form with `other` and returns the reduced result.
    ///
    /// This is the primary group operation $f_1 \circ f_2$ of the class group $\text{Cl}(D)$.
    /// Internally calls [`Form::nucomp`] followed by [`Form::reduce`].
    pub fn compose(&self, other: &Form, d_disc: &BigInt) -> Form {
        let l = isqrt_fourth(&d_disc.abs());
        let mut form = self.nucomp(other, d_disc, &l);
        form.reduce(d_disc);
        form
    }

    /// Squares this form and returns the reduced result.
    ///
    /// Equivalent to `self.compose(self, d_disc)` but uses the faster
    /// [`Form::nudupl`] specialization for self-composition.
    pub fn square(&self, d_disc: &BigInt) -> Form {
        let l = isqrt_fourth(&d_disc.abs());
        let mut form = self.nudupl(d_disc, &l);
        form.reduce(d_disc);
        form
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduction_valid_negative_discriminant() {
        let d = BigInt::from(-71);
        let mut form = Form::new(BigInt::from(2), BigInt::from(1), BigInt::from(9));
        form.reduce(&d);
        assert_eq!(form.a, BigInt::from(2));
        assert_eq!(form.b, BigInt::from(1));
        assert_eq!(form.c, BigInt::from(9)); // Already in reduced form
    }

    #[test]
    fn test_compose_and_square() {
        let d = BigInt::from(-71);
        let id = Form::identity(&d); // (1, 1, 18)
        let f2 = Form::new(BigInt::from(2), BigInt::from(1), BigInt::from(9));

        // Compose with identity must equal self
        let comp1 = id.compose(&f2, &d);
        assert_eq!(comp1, f2);

        // Square
        let sq = f2.square(&d);
        assert_eq!(sq, Form::new(BigInt::from(4), BigInt::from(-3), BigInt::from(5)));

        // Compose f2 with itself must equal square
        let comp2 = f2.compose(&f2, &d);
        assert_eq!(comp2, sq);

        // f2 * f2^-1 must equal identity
        let f2_inv = Form::new(f2.a.clone(), -f2.b.clone(), f2.c.clone());
        let comp3 = f2.compose(&f2_inv, &d);
        assert_eq!(comp3, id);
    }
}
