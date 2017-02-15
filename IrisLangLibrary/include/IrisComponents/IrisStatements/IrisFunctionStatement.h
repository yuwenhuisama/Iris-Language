#ifndef _H_IRISSTATEMENT_
#define _H_IRISSTATEMENT_

#include "IrisStatement.h"

class IrisFunctionHeader;
class IrisBlock;
class IrisFunctionStatement :
	public IrisStatement
{
protected:
public:
	IrisFunctionHeader* m_pFunctionHeader = nullptr;
	IrisBlock* m_pWithBlock = nullptr;
	IrisBlock* m_pWithoutBlock = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisFunctionStatement(IrisFunctionHeader* pFunctionHeader, IrisBlock* pWithBlock, IrisBlock* pWithoutBlock, IrisBlock* pBlock);
	virtual ~IrisFunctionStatement();

	virtual bool Validate() override;
};

#endif