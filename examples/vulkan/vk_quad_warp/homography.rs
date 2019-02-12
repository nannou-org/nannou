/*
 *  Created by Elliot Woods on 26/11/2010.
 *  Edited by Krisjanis Rijnieks on 23/01/2016
 *  Ported by Tom Gowan on 31/01/2019
 *
 *  Based entirely on arturo castro's homography implementation
 *  Created 08/01/2010, arturo castro
 *
 *      http://www.openframeworks.cc/forum/viewtopic.php?f=9&t=3121
 */

pub fn find_homography(src: [[f32; 2]; 4], dst: [[f32; 2]; 4]) -> [f32; 16] {
    let mut p: [f32; 72] = [
        -src[0][0],
        -src[0][1],
        -1.0,
        0.0,
        0.0,
        0.0,
        src[0][0] * dst[0][0],
        src[0][1] * dst[0][0],
        -dst[0][0], // h11
        0.0,
        0.0,
        0.0,
        -src[0][0],
        -src[0][1],
        -1.0,
        src[0][0] * dst[0][1],
        src[0][1] * dst[0][1],
        -dst[0][1], // h12
        -src[1][0],
        -src[1][1],
        -1.0,
        0.0,
        0.0,
        0.0,
        src[1][0] * dst[1][0],
        src[1][1] * dst[1][0],
        -dst[1][0], // h13
        0.0,
        0.0,
        0.0,
        -src[1][0],
        -src[1][1],
        -1.0,
        src[1][0] * dst[1][1],
        src[1][1] * dst[1][1],
        -dst[1][1], // h21
        -src[2][0],
        -src[2][1],
        -1.0,
        0.0,
        0.0,
        0.0,
        src[2][0] * dst[2][0],
        src[2][1] * dst[2][0],
        -dst[2][0], // h22
        0.0,
        0.0,
        0.0,
        -src[2][0],
        -src[2][1],
        -1.0,
        src[2][0] * dst[2][1],
        src[2][1] * dst[2][1],
        -dst[2][1], // h23
        -src[3][0],
        -src[3][1],
        -1.0,
        0.0,
        0.0,
        0.0,
        src[3][0] * dst[3][0],
        src[3][1] * dst[3][0],
        -dst[3][0], // h31
        0.0,
        0.0,
        0.0,
        -src[3][0],
        -src[3][1],
        -1.0,
        src[3][0] * dst[3][1],
        src[3][1] * dst[3][1],
        -dst[3][1], // h32
    ];

    gaussian_elimination(&mut p);

    // gaussian elimination gives the results of the equation system
    // in the last column of the original matrix.
    // opengl needs the transposed 4x4 matrix:
    [
        p[8],
        p[3 * 9 + 8],
        0.0,
        p[6 * 9 + 8], // h11  h21 0 h31
        p[1 * 9 + 8],
        p[4 * 9 + 8],
        0.0,
        p[7 * 9 + 8], // h12  h22 0 h32
        0.0,
        0.0,
        0.0,
        0.0, // 0    0   0 0
        p[2 * 9 + 8],
        p[5 * 9 + 8],
        0.0,
        1.0,
    ] // h13  h23 0 h33
}

fn gaussian_elimination(a: &mut [f32; 72]) {
    let mut i = 0;
    let mut j = 0;
    let n = 9;
    let m = n - 1;
    while i < m && j < n {
        // Find pivot in column j, starting in row i:
        let mut maxi = i;
        for k in (i + 1)..m {
            if (a[k * n + j]).abs() > (a[maxi * n + j]).abs() {
                maxi = k;
            }
        }
        if a[maxi * n + j] != 0.0 {
            //swap rows i and maxi, but do not change the value of i
            if i != maxi {
                for k in 0..n {
                    let aux = a[i * n + k];
                    a[i * n + k] = a[maxi * n + k];
                    a[maxi * n + k] = aux;
                }
            }
            //Now A[i,j] will contain the old value of A[maxi,j].
            //divide each entry in row i by A[i,j]
            let a_ij = a[i * n + j];
            for k in 0..n {
                a[i * n + k] /= a_ij;
            }
            //Now A[i,j] will have the value 1.
            for u in (i + 1)..m {
                //subtract A[u,j] * row i from row u
                let a_uj = a[u * n + j];
                for k in 0..n {
                    a[u * n + k] -= a_uj * a[i * n + k];
                }
                //Now A[u,j] will be 0, since A[u,j] - A[i,j] * A[u,j] = A[u,j] - 1 * A[u,j] = 0.
            }

            i += 1;
        }
        j += 1;
    }

    //back substitution
    for i in (0..(m - 1)).rev() {
        for j in (i + 1)..(n - 1) {
            a[i * n + m] -= a[i * n + j] * a[j * n + m];
        }
    }
}
