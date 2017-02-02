#ifndef _H_IRISARRAYTAG_
#define _H_IRISARRAYTAG_

#include "IrisDevHeader.h"

#include <vector>
using namespace std;

#if IR_USE_MEM_POOL
class IrisArrayTag : public IrisObjectMemoryPoolInterface<IrisArrayTag, POOLID_IrisArrayTag>
#else
class IrisArrayTag
#endif
{
private:
	vector<IrisValue> m_vcValues;

public:

	void Initialize(IrisValues* pValues);

	/**
	 * return the object of index.
	 */
	IrisValue At(int nIndex);

	/**
	 * set an object to index of array
	 */
	IrisValue Set(int nIndex, const IrisValue& ivValue);
	
	/*
	 * push an object to the end of this array
	 */
	IrisValue Push(const IrisValue& ivValue);
	/*
	 * pop the last object 
	 */
	IrisValue Pop();

	/*
	 *  return array size
	 */
	int Size();

	/*
	 * return if is empty array
	 */
	bool Empty();

	/**
	 * return the index of a elemet
	 * if not exist , return -1
	 */
	int IndexOf(IrisValue & aRef);

	/**
	 * return if an Object exists in this Array
	 */
	bool Include(IrisValue & aRef);

	/*
	 * merge two array
	 */
	IrisArrayTag & Merge(IrisArrayTag & aRef);



	IrisArrayTag();
	~IrisArrayTag();

	friend class IrisArray;
	friend class IrisArrayIterator;
	friend class IrisStatement;
};

#endif