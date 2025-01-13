#include "mandelbrot.h"

#include <immintrin.h>

#include <algorithm>
#include <chrono>
#include <functional>
#include <iostream>
#include <string>
#include <thread>
#include <vector>
#include <cstdlib>
#include <numeric>

int mandelbrot_calc_base(float x, float y)
{
    auto re = x;
    auto im = y;

    for (auto i = 0; i < LOOP; i++)
    {
        float re2 = re * re;
        float im2 = im * im;

        // verify if f(z) diverges to infinity
        if (re2 + im2 > 4.0f)
            return i;

        im = 2 * re * im + y;
        re = re2 - im2 + x;
    }

    return LOOP;
}

void naive_mandelbrot(int width, int height, int *plot)
{
    float dx = (X_END - X_START) / (width - 1);
    float dy = (Y_END - Y_START) / (height - 1);

    for (int i = 0; i < height; i++)
    {
        for (int j = 0; j < width; j++)
        {
            float x = X_START + j * dx; // real value
            float y = Y_END - i * dy;   // imaginary value

            auto result = mandelbrot_calc_base(x, y);
            plot[i * width + j] = result;
        }
    }
}

__m128i sse_mandelbrot_calc_base(__m128 x, __m128 y)
{
    __m128 re = x;
    __m128 im = y;
    __m128i res = _mm_setzero_si128();

    for (int i = 0; i < LOOP; i++)
    {
        __m128 re2 = _mm_mul_ps(re, re);
        __m128 im2 = _mm_mul_ps(im, im);

        __m128 mask = _mm_cmple_ps(_mm_add_ps(re2, im2), _mm_set_ps1(4.0f));
        res = _mm_add_epi32(res, _mm_and_si128(_mm_castps_si128(mask), _mm_set1_epi32(1)));
        if ((_mm_movemask_ps(mask) & 0x0F) == 0)
        {
            break;
        }

        im = _mm_add_ps(_mm_mul_ps(_mm_set1_ps(2.0f), _mm_mul_ps(re, im)), y);
        re = _mm_add_ps(_mm_sub_ps(re2, im2), x);
    }
    return res;
}

void _sse_optimized_mandelbrot(int width, int height, int *plot, int start_row, int end_row)
{
    auto start = std::chrono::steady_clock::now();
    // std::cout << "start_row: " << start_row << ", end_row: " << end_row << std::endl;
    float dx = (X_END - X_START) / (width - 1);
    float dy = (Y_END - Y_START) / (height - 1);

    for (int row = start_row; row < end_row; row++)
    {
        for (int col = 0; col < width; col += 4)
        {
            __m128 x = _mm_setr_ps(X_START + (col + 0) * dx,
                                   X_START + (col + 1) * dx,
                                   X_START + (col + 2) * dx,
                                   X_START + (col + 3) * dx);
            __m128 y = _mm_set1_ps(Y_END - row * dy);

            __m128i res = sse_mandelbrot_calc_base(x, y);
            _mm_storeu_si128((__m128i *)&plot[row * width + col], res);
        }
    }
    auto end = std::chrono::steady_clock::now();
    std::chrono::duration<double> diff = end - start;
    std::cout << "start_row: " << start_row << ", end_row: " << end_row << ", duration: " << diff.count() << std::endl;
}

const float percentages[] = {0.43f, 0.07f, 0.07f, 0.43f};

std::vector<int> distribute_workload(int height)
{

    std::vector<int> n_rows;
    int remaining = height;

    for (int i = 0; i < 3; ++i)
    {
        int size = static_cast<int>(height * percentages[i]);
        n_rows.push_back(size);
        remaining -= size;
    }

    n_rows.push_back(remaining);
    return n_rows;
}

void sse_optimized_mandelbrot(int width, int height, int *plot)
{
    int num_threads = 4;
    std::vector<int> workload = distribute_workload(height);
    std::vector<std::thread> thread_pool;

    int start = 0;

    for (int i = 0; i < num_threads; i++)
    {
        int n_rows = workload[i];
        // int n_rows = height / 4;
        thread_pool.emplace_back(std::thread(_sse_optimized_mandelbrot, width, height, plot, start, start + n_rows));
        start += n_rows;
    }

    for (std::thread &thread : thread_pool)
    {
        if (thread.joinable())
        {
            thread.join();
        }
    }
}

void _sse_optimized_mandelbrot_shuffle(int width, int height, int *plot, const int *rows, int n_rows)
{
    float dx = (X_END - X_START) / (width - 1);
    float dy = (Y_END - Y_START) / (height - 1);

    for (int i = 0; i < n_rows; i++)
    {
        int row = rows[i];
        for (int col = 0; col < width; col += 4)
        {
            __m128 x = _mm_setr_ps(X_START + (col + 0) * dx,
                                   X_START + (col + 1) * dx,
                                   X_START + (col + 2) * dx,
                                   X_START + (col + 3) * dx);
            __m128 y = _mm_set1_ps(Y_END - row * dy);

            __m128i res = sse_mandelbrot_calc_base(x, y);
            _mm_storeu_si128((__m128i *)&plot[row * width + col], res);
        }
    }
}

void sse_optimized_mandelbrot_shuffle(int width, int height, int *plot)
{
    int num_threads = 4;
    int n_rows = height / num_threads;
    std::vector<std::thread> thread_pool;
    std::vector<int> rows(height);
    std::iota(rows.begin(), rows.end(), 0);
    std::random_shuffle(rows.begin(), rows.end());

    for (int i = 0; i < num_threads; i++)
    {
        auto start = rows.begin() + i * n_rows;
        auto end = start + n_rows;
        std::vector<int> *workload = new std::vector<int>(start, end);
        thread_pool.emplace_back(std::thread(_sse_optimized_mandelbrot_shuffle, width, height, plot, workload->data(), n_rows));
    }

    for (std::thread &thread : thread_pool)
    {
        if (thread.joinable())
        {
            thread.join();
        }
    }
}

#ifdef __cplusplus
extern "C"
{
#endif
    void mandelbrot(int width, int height, int *plot)
    {
        sse_optimized_mandelbrot_shuffle(width, height, plot);
    }

#ifdef __cplusplus
}
#endif
