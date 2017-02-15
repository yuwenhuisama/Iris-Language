#ifndef _H_IRISGETTERSTAMENT_
#define _H_IRISGETTERSTAMENT_

#include "IrisStatement.h"

class IrisIdentifier;
class IrisBlock;
class IrisGetterStatement :
	public IrisStatement
{
public:
	IrisIdentifier* m_pGetteredVariable = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisGetterStatement(IrisIdentifier* pGetteredVariable, IrisBlock* pBlock);
	virtual ~IrisGetterStatement();

	virtual bool Validate() override;
};

#endif